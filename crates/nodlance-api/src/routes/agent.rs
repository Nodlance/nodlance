//! Agent ingestion route handlers.

use axum::{Json, extract::State};
use nodlance_domain::{
    AgentRepository, AgentStatus, DomainError, HeartbeatPayload, HeartbeatRepository,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

#[derive(Serialize)]
pub struct HeartbeatResponse {
    pub status: &'static str,
    pub host_id: Uuid,
}

pub async fn receive_heartbeat(
    State(state): State<AppState>,
    Json(payload): Json<HeartbeatPayload>,
) -> Result<Json<HeartbeatResponse>, ApiError> {
    // 1. Verify that agent is enrolled and active
    let agent = state
        .agent_repo
        .get_by_fingerprint(&payload.fingerprint)
        .await?
        .ok_or(ApiError(DomainError::Unauthorized))?;

    if agent.status != AgentStatus::Enrolled {
        return Err(ApiError(DomainError::Unauthorized));
    }

    // 2. Record heartbeat and telemetry
    state
        .heartbeat_repo
        .record_heartbeat(
            agent.host_id,
            &payload.agent_version,
            payload.metrics.as_ref(),
        )
        .await?;

    Ok(Json(HeartbeatResponse {
        status: "ok",
        host_id: agent.host_id,
    }))
}
