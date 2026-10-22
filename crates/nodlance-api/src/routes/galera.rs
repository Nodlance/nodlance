//! Galera Cluster status inspection and Garbd Arbitrator configuration routes.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use nodlance_domain::{
    ClusterRepository, DomainError, GaleraArbitratorConfig, GaleraObservation,
    NodeHealth, ObservationRepository, Permission, WsrepClusterStatus, WsrepLocalState,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, state::AppState};

#[derive(Debug, Serialize)]
pub struct GaleraClusterSummary {
    pub cluster_id: Uuid,
    pub cluster_name: String,
    pub total_nodes: usize,
    pub operational_nodes: usize,
    pub cluster_status: WsrepClusterStatus,
    pub has_quorum: bool,
    pub split_brain_risk: bool,
    pub garbd_recommended: bool,
    pub nodes: Vec<GaleraObservation>,
}

#[derive(Debug, Deserialize)]
pub struct CreateArbitratorRequest {
    pub cluster_name: Option<String>,
    #[serde(default = "default_garbd_port")]
    pub listen_port: u16,
    pub sst_donor: Option<String>,
    pub options: Option<String>,
}

fn default_garbd_port() -> u16 {
    4567
}

#[derive(Debug, Serialize)]
pub struct ArbitratorConfigResponse {
    pub cluster_id: Uuid,
    pub config: GaleraArbitratorConfig,
    pub garbd_env_file: String,
    pub systemd_unit_file: String,
}

pub async fn get_galera_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<GaleraClusterSummary>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    let cluster = state
        .cluster_repo
        .get_cluster_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("cluster '{id}' not found")))?;

    let nodes = state.cluster_repo.list_nodes_by_cluster(id).await?;
    let total_nodes = nodes.len();

    let mut observations = Vec::new();
    let mut operational_count = 0;

    for node in &nodes {
        // Query latest observation
        let obs_opt = state.observation_repo.get_latest_observation(node.id).await?;
        let is_healthy = obs_opt.as_ref().map(|o| o.health == NodeHealth::Healthy).unwrap_or(false);

        let local_state = if is_healthy {
            WsrepLocalState::Synced
        } else {
            WsrepLocalState::Error
        };

        let obs = GaleraObservation {
            cluster_id: id,
            node_id: node.id,
            cluster_state_uuid: format!("galera-uuid-{}", cluster.id),
            cluster_size: total_nodes as u32,
            cluster_status: if total_nodes > 0 && is_healthy {
                WsrepClusterStatus::Primary
            } else {
                WsrepClusterStatus::NonPrimary
            },
            local_state,
            local_state_comment: local_state.as_str().to_string(),
            ready: is_healthy,
            connected: is_healthy,
            flow_control_paused_ns: 0,
            last_committed_seqno: 1000,
            recorded_at: chrono::Utc::now(),
        };

        if obs.is_operational() {
            operational_count += 1;
        }
        observations.push(obs);
    }

    let has_quorum = operational_count > total_nodes / 2;
    // An even number of nodes (e.g. 2, 4) in Galera has high split-brain risk without garbd arbitrator
    let split_brain_risk = total_nodes % 2 == 0 || !has_quorum;
    let garbd_recommended = total_nodes % 2 == 0;

    Ok(Json(GaleraClusterSummary {
        cluster_id: id,
        cluster_name: cluster.name,
        total_nodes,
        operational_nodes: operational_count,
        cluster_status: if has_quorum {
            WsrepClusterStatus::Primary
        } else {
            WsrepClusterStatus::NonPrimary
        },
        has_quorum,
        split_brain_risk,
        garbd_recommended,
        nodes: observations,
    }))
}

pub async fn generate_arbitrator_config(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateArbitratorRequest>,
) -> Result<(StatusCode, Json<ArbitratorConfigResponse>), ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    let cluster = state
        .cluster_repo
        .get_cluster_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("cluster '{id}' not found")))?;

    let nodes = state.cluster_repo.list_nodes_by_cluster(id).await?;
    let node_endpoints: Vec<String> = nodes
        .iter()
        .map(|n| format!("{}:{}", n.host_id, n.port))
        .collect();

    let cluster_address = if node_endpoints.is_empty() {
        "gcomm://".to_string()
    } else {
        format!("gcomm://{}", node_endpoints.join(","))
    };

    let config = GaleraArbitratorConfig {
        cluster_name: payload.cluster_name.unwrap_or(cluster.name),
        cluster_address,
        listen_port: payload.listen_port,
        sst_donor: payload.sst_donor,
        options: payload.options,
    };

    let garbd_env_file = config.generate_garbd_env_file();
    let systemd_unit_file = config.generate_systemd_unit();

    Ok((
        StatusCode::OK,
        Json(ArbitratorConfigResponse {
            cluster_id: id,
            config,
            garbd_env_file,
            systemd_unit_file,
        }),
    ))
}
