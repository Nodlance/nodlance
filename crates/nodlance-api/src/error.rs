//! API error representation and HTTP mapping.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use nodlance_auth::AuthError;
use nodlance_domain::DomainError;
use serde::Serialize;

#[derive(Debug)]
pub struct ApiError(pub DomainError);

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        Self(err)
    }
}

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        Self(DomainError::from(err))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self.0 {
            DomainError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            DomainError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            DomainError::Validation(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg),
            DomainError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "authentication required".to_string(),
            ),
            DomainError::Forbidden(perm) => (
                StatusCode::FORBIDDEN,
                "forbidden",
                format!("permission denied: requires {perm}"),
            ),
            DomainError::Internal(msg) => {
                tracing::error!(error = %msg, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "an internal error occurred".to_string(),
                )
            }
        };

        let envelope = ErrorEnvelope {
            error: ErrorBody { code, message },
        };

        (status, Json(envelope)).into_response()
    }
}
