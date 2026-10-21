//! User management route handlers.

use axum::{Json, extract::State, http::StatusCode};
use nodlance_auth::hash_password;
use nodlance_domain::{
    AuditRepository, DomainError, NewAuditEvent, NewUser, Permission, User, UserRepository,
};

use crate::{auth::AuthUser, error::ApiError, state::AppState};

pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<User>>, ApiError> {
    auth.0.require_permission(Permission::UserRead)?;
    let users = state.user_repo.list().await?;
    Ok(Json(users))
}

pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<NewUser>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    auth.0.require_permission(Permission::UserWrite)?;

    if payload.username.trim().is_empty() {
        return Err(ApiError(DomainError::Validation(
            "username cannot be empty".to_string(),
        )));
    }
    if payload.password.len() < 8 {
        return Err(ApiError(DomainError::Validation(
            "password must be at least 8 characters".to_string(),
        )));
    }

    let password_hash = hash_password(&payload.password)?;
    let user = state.user_repo.create(&payload, &password_hash).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "user.create".to_string(),
            resource_type: "user".to_string(),
            resource_id: Some(user.id),
            details: Some(serde_json::json!({
                "username": user.username,
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::CREATED, Json(user)))
}
