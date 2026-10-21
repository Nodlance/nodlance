//! Database clusters and topology route handlers.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use nodlance_domain::{
    AuditRepository, Cluster, ClusterRepository, ClusterTopology, DatabaseNode, DomainError,
    NewAuditEvent, NewCluster, NewDatabaseNode, ObservationRepository, Permission,
};
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, state::AppState};

pub async fn list_clusters(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<Cluster>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let clusters = state.cluster_repo.list_clusters().await?;
    Ok(Json(clusters))
}

pub async fn create_cluster(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<NewCluster>,
) -> Result<(StatusCode, Json<Cluster>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    if payload.name.trim().is_empty() {
        return Err(ApiError(DomainError::Validation(
            "cluster name cannot be empty".to_string(),
        )));
    }

    let cluster = state.cluster_repo.create_cluster(&payload).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "cluster.create".to_string(),
            resource_type: "cluster".to_string(),
            resource_id: Some(cluster.id),
            details: Some(serde_json::json!({
                "name": cluster.name,
                "engine": cluster.engine.as_str(),
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::CREATED, Json(cluster)))
}

pub async fn get_cluster(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Cluster>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let cluster = state
        .cluster_repo
        .get_cluster_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("cluster not found: {id}")))?;

    Ok(Json(cluster))
}

pub async fn add_node_to_cluster(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(cluster_id): Path<Uuid>,
    Json(mut payload): Json<NewDatabaseNode>,
) -> Result<(StatusCode, Json<DatabaseNode>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    payload.cluster_id = cluster_id;

    if payload.name.trim().is_empty() {
        return Err(ApiError(DomainError::Validation(
            "node name cannot be empty".to_string(),
        )));
    }

    let node = state.cluster_repo.create_node(&payload).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "cluster.node.create".to_string(),
            resource_type: "database_node".to_string(),
            resource_id: Some(node.id),
            details: Some(serde_json::json!({
                "cluster_id": cluster_id,
                "node_name": node.name,
                "port": node.port,
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::CREATED, Json(node)))
}

pub async fn get_cluster_topology(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ClusterTopology>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let topology = state.observation_repo.get_cluster_topology(id).await?;
    Ok(Json(topology))
}
