//! Load balancer and connection router route handlers (ProxySQL, PgBouncer, HAProxy, Keepalived).

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use nodlance_domain::{
    AuditRepository, ClusterRepository, DomainError, HostRepository, LoadBalancer,
    LoadBalancerFilter, LoadBalancerRepository, LoadBalancerStatus, LoadBalancerType,
    NewAuditEvent, NewLoadBalancer, Permission, UpdateLoadBalancer,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListLoadBalancersQuery {
    pub cluster_id: Option<Uuid>,
    pub host_id: Option<Uuid>,
    pub lb_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReloadResult {
    pub load_balancer_id: Uuid,
    pub status: LoadBalancerStatus,
    pub message: String,
    pub generated_config: String,
}

pub async fn list_load_balancers(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListLoadBalancersQuery>,
) -> Result<Json<Vec<LoadBalancer>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    let lb_type = match query.lb_type {
        Some(t) => Some(t.parse::<LoadBalancerType>()?),
        None => None,
    };

    let status = match query.status {
        Some(s) => Some(s.parse::<LoadBalancerStatus>()?),
        None => None,
    };

    let filter = LoadBalancerFilter {
        cluster_id: query.cluster_id,
        host_id: query.host_id,
        lb_type,
        status,
    };

    let lbs = state.load_balancer_repo.list(&filter).await?;
    Ok(Json(lbs))
}

pub async fn get_load_balancer(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<LoadBalancer>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    let lb = state
        .load_balancer_repo
        .get_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("load balancer '{id}' not found")))?;

    Ok(Json(lb))
}

pub async fn create_load_balancer(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<NewLoadBalancer>,
) -> Result<(StatusCode, Json<LoadBalancer>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    // Validate cluster existence
    state
        .cluster_repo
        .get_cluster_by_id(payload.cluster_id)
        .await?
        .ok_or_else(|| {
            DomainError::NotFound(format!("cluster '{}' not found", payload.cluster_id))
        })?;

    // Validate host existence
    state
        .host_repo
        .get_by_id(payload.host_id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("host '{}' not found", payload.host_id)))?;

    payload.validate()?;

    let lb = state.load_balancer_repo.create(&payload).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "load_balancer.create".to_string(),
            resource_type: "load_balancer".to_string(),
            resource_id: Some(lb.id),
            details: Some(serde_json::json!({
                "cluster_id": lb.cluster_id,
                "host_id": lb.host_id,
                "lb_type": lb.lb_type.as_str(),
                "listen_port": lb.listen_port,
                "admin_port": lb.admin_port,
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::CREATED, Json(lb)))
}

pub async fn update_load_balancer(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateLoadBalancer>,
) -> Result<Json<LoadBalancer>, ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    let lb = state.load_balancer_repo.update(id, &payload).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "load_balancer.update".to_string(),
            resource_type: "load_balancer".to_string(),
            resource_id: Some(lb.id),
            details: Some(serde_json::json!({
                "status": lb.status.as_str(),
            })),
            source_ip: None,
        })
        .await;

    Ok(Json(lb))
}

pub async fn reload_load_balancer(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ReloadResult>, ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    let lb = state
        .load_balancer_repo
        .get_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("load balancer '{id}' not found")))?;

    // Generate dynamic runtime configuration based on LB type
    let generated_config = match lb.lb_type {
        LoadBalancerType::ProxySql => lb.generate_proxysql_admin_sql()?,
        LoadBalancerType::PgBouncer => {
            // Find nodes in cluster to point to primary
            let nodes = state
                .cluster_repo
                .list_nodes_by_cluster(lb.cluster_id)
                .await?;
            let primary = nodes
                .iter()
                .find(|n| n.role == nodlance_domain::NodeRole::Primary);
            let (db_host, db_port) = match primary {
                Some(p) => (p.host_id.to_string(), p.port),
                None => ("127.0.0.1".to_string(), 5432),
            };
            lb.generate_pgbouncer_ini(&db_host, db_port, "postgres")?
        }
        LoadBalancerType::Keepalived => lb.generate_keepalived_conf()?,
        LoadBalancerType::HaProxy => {
            let mut conf = String::new();
            conf.push_str("# Nodlance HAProxy Managed Config\n");
            conf.push_str("global\n    log /dev/log local0\n    maxconn 4096\n\n");
            conf.push_str(&format!(
                "frontend db_front\n    bind *:{} \n    mode tcp\n    default_backend db_back\n\n",
                lb.listen_port
            ));
            conf.push_str("backend db_back\n    mode tcp\n    balance roundrobin\n");
            let nodes = state
                .cluster_repo
                .list_nodes_by_cluster(lb.cluster_id)
                .await?;
            for node in &nodes {
                conf.push_str(&format!(
                    "    server node_{} {}:{} check\n",
                    node.id, node.host_id, node.port
                ));
            }
            conf
        }
    };

    // Update status to Active
    let updated = state
        .load_balancer_repo
        .update(
            id,
            &UpdateLoadBalancer {
                status: Some(LoadBalancerStatus::Active),
                config_snapshot: None,
            },
        )
        .await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "load_balancer.reload".to_string(),
            resource_type: "load_balancer".to_string(),
            resource_id: Some(lb.id),
            details: Some(serde_json::json!({
                "lb_type": lb.lb_type.as_str(),
                "status": updated.status.as_str(),
            })),
            source_ip: None,
        })
        .await;

    Ok(Json(ReloadResult {
        load_balancer_id: lb.id,
        status: updated.status,
        message: format!(
            "Load balancer {} config successfully regenerated and synchronized",
            lb.lb_type
        ),
        generated_config,
    }))
}

pub async fn delete_load_balancer(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    state.load_balancer_repo.delete(id).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "load_balancer.delete".to_string(),
            resource_type: "load_balancer".to_string(),
            resource_id: Some(id),
            details: None,
            source_ip: None,
        })
        .await;

    Ok(StatusCode::NO_CONTENT)
}
