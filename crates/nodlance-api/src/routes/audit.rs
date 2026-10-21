//! Audit log query route handler.

use axum::{
    Json,
    extract::{Query, State},
};
use nodlance_domain::{AuditEvent, AuditFilter, AuditRepository, Permission};

use crate::{auth::AuthUser, error::ApiError, state::AppState};

pub async fn list_audit_events(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(filter): Query<AuditFilter>,
) -> Result<Json<Vec<AuditEvent>>, ApiError> {
    auth.0.require_permission(Permission::AuditRead)?;
    let events = state.audit_repo.list(&filter).await?;
    Ok(Json(events))
}
