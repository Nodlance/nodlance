//! Host inventory and agent enrollment route handlers.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use nodlance_domain::{
    AgentIdentity, AgentRepository, AuditRepository, DomainError, EnrollmentRequest, Host,
    HostFilter, HostRepository, NewAuditEvent, NewHost, Permission, UpdateHost,
};
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, state::AppState};

pub async fn list_hosts(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(filter): Query<HostFilter>,
) -> Result<Json<Vec<Host>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let hosts = state.host_repo.list(&filter).await?;
    Ok(Json(hosts))
}

pub async fn create_host(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<NewHost>,
) -> Result<(StatusCode, Json<Host>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    if payload.hostname.trim().is_empty() {
        return Err(ApiError(DomainError::Validation(
            "hostname cannot be empty".to_string(),
        )));
    }

    let host = state.host_repo.create(auth.0.user_id, &payload).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "host.create".to_string(),
            resource_type: "host".to_string(),
            resource_id: Some(host.id),
            details: Some(serde_json::json!({
                "hostname": host.hostname,
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::CREATED, Json(host)))
}

pub async fn get_host(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Host>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let host = state
        .host_repo
        .get_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("host not found: {id}")))?;

    Ok(Json(host))
}

pub async fn update_host(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateHost>,
) -> Result<Json<Host>, ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;
    let host = state.host_repo.update(id, &payload).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "host.update".to_string(),
            resource_type: "host".to_string(),
            resource_id: Some(host.id),
            details: Some(serde_json::json!({
                "hostname": host.hostname,
                "status": host.status,
            })),
            source_ip: None,
        })
        .await;

    Ok(Json(host))
}

pub async fn decommission_host(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Host>, ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;
    let host = state.host_repo.decommission(id).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "host.decommission".to_string(),
            resource_type: "host".to_string(),
            resource_id: Some(id),
            details: None,
            source_ip: None,
        })
        .await;

    Ok(Json(host))
}

pub async fn enroll_agent(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<EnrollmentRequest>,
) -> Result<(StatusCode, Json<AgentIdentity>), ApiError> {
    auth.0.require_permission(Permission::AgentWrite)?;

    if payload.fingerprint.trim().is_empty() {
        return Err(ApiError(DomainError::Validation(
            "fingerprint cannot be empty".to_string(),
        )));
    }

    // Verify host exists
    state
        .host_repo
        .get_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("host not found: {id}")))?;

    let agent = state.agent_repo.enroll(id, &payload.fingerprint).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "agent.enroll".to_string(),
            resource_type: "agent".to_string(),
            resource_id: Some(agent.id),
            details: Some(serde_json::json!({
                "host_id": id,
                "fingerprint": agent.fingerprint,
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::CREATED, Json(agent)))
}

pub async fn revoke_agent(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentIdentity>, ApiError> {
    auth.0.require_permission(Permission::AgentWrite)?;

    let agent = state.agent_repo.revoke(id, auth.0.user_id).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "agent.revoke".to_string(),
            resource_type: "agent".to_string(),
            resource_id: Some(agent.id),
            details: Some(serde_json::json!({
                "host_id": id,
            })),
            source_ip: None,
        })
        .await;

    Ok(Json(agent))
}
