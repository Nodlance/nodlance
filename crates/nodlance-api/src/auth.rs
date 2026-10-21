//! Authentication extractor and context helpers.

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use nodlance_auth::{AuthContext, validate_token};
use nodlance_domain::DomainError;

use crate::{error::ApiError, state::AppState};

/// Request extractor that parses and validates Bearer JWT tokens.
pub struct AuthUser(pub AuthContext);

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(ApiError(DomainError::Unauthorized))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError(DomainError::Unauthorized))?;

        let claims = validate_token(token, &app_state.jwt_secret)
            .map_err(|e| ApiError(DomainError::from(e)))?;

        Ok(AuthUser(AuthContext::from_claims(claims)))
    }
}
