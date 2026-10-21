//! Health probes and version info route handlers.

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub name: &'static str,
    pub version: &'static str,
    pub stage: &'static str,
}

pub async fn health_live() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub async fn health_ready(State(state): State<AppState>) -> Response {
    if let Some(pool) = &state.db {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => (StatusCode::OK, Json(HealthResponse { status: "ready" })).into_response(),
            Err(e) => {
                tracing::warn!(error = %e, "readiness check database probe failed");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(HealthResponse {
                        status: "database_unavailable",
                    }),
                )
                    .into_response()
            }
        }
    } else {
        (StatusCode::OK, Json(HealthResponse { status: "ready" })).into_response()
    }
}

pub async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        name: "Nodlance",
        version: env!("CARGO_PKG_VERSION"),
        stage: "inventory_and_enrollment",
    })
}
