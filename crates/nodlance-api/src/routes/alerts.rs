//! Alerting and maintenance window route handlers.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use nodlance_domain::{
    AlertIncident, AlertRepository, AlertRule, AuditRepository, MaintenanceWindow, NewAuditEvent,
    NewMaintenanceWindow, Permission,
};
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, state::AppState};

pub async fn list_alert_rules(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<AlertRule>>, ApiError> {
    auth.0.require_permission(Permission::AuditRead)?;
    let rules = state.alert_repo.list_rules().await?;
    Ok(Json(rules))
}

pub async fn list_active_alerts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<AlertIncident>>, ApiError> {
    auth.0.require_permission(Permission::AuditRead)?;
    let incidents = state.alert_repo.list_active_incidents().await?;
    Ok(Json(incidents))
}

pub async fn acknowledge_alert(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AlertIncident>, ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;
    let incident = state.alert_repo.acknowledge_incident(id).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "alert.acknowledge".to_string(),
            resource_type: "alert_incident".to_string(),
            resource_id: Some(id),
            details: Some(serde_json::json!({
                "title": incident.title,
                "status": incident.status.as_str(),
            })),
            source_ip: None,
        })
        .await;

    Ok(Json(incident))
}

pub async fn create_maintenance_window(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<NewMaintenanceWindow>,
) -> Result<(StatusCode, Json<MaintenanceWindow>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;
    let window = state.alert_repo.create_maintenance_window(&payload).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "maintenance_window.create".to_string(),
            resource_type: "maintenance_window".to_string(),
            resource_id: Some(window.id),
            details: Some(serde_json::json!({
                "title": window.title,
                "starts_at": window.starts_at,
                "ends_at": window.ends_at,
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::CREATED, Json(window)))
}
