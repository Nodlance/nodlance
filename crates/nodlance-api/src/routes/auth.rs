//! Authentication route handlers.

use axum::{Json, extract::State};
use nodlance_auth::{generate_token, verify_password};
use nodlance_domain::{DomainError, Permission, UserRepository};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub is_active: bool,
    pub permissions: Vec<Permission>,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let user = state
        .user_repo
        .get_by_username(&payload.username)
        .await?
        .ok_or(ApiError(DomainError::Unauthorized))?;

    if !user.is_active {
        return Err(ApiError(DomainError::Unauthorized));
    }

    let valid = verify_password(&payload.password, &user.password_hash)?;
    if !valid {
        return Err(ApiError(DomainError::Unauthorized));
    }

    let permissions = state.user_repo.get_user_permissions(user.id).await?;
    let ttl_secs = 24 * 3600;
    let token = generate_token(
        user.id,
        &user.username,
        &permissions,
        &state.jwt_secret,
        ttl_secs,
    )?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer",
        expires_in: ttl_secs,
        user: UserInfo {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            is_active: user.is_active,
            permissions,
        },
    }))
}
