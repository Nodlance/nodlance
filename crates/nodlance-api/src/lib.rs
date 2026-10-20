//! Initial HTTP API for Nodlance. Only health and version endpoints exist today.

use axum::{Json, Router, routing::get};
use serde::Serialize;

pub fn build_app() -> Router {
    Router::new()
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .route("/api/v1/version", get(version))
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health_live() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// The scaffold has no external runtime dependencies yet. Update this check
/// when metadata storage and other required dependencies are introduced.
async fn health_ready() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ready" })
}

#[derive(Debug, Serialize)]
struct VersionResponse {
    name: &'static str,
    version: &'static str,
    stage: &'static str,
}

async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        name: "Nodlance",
        version: env!("CARGO_PKG_VERSION"),
        stage: "scaffold",
    })
}

#[cfg(test)]
mod tests {
    use super::build_app;
    use axum::{body::Body, http::{Request, StatusCode}};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn liveness_endpoint_returns_ok() {
        let request = Request::builder()
            .uri("/health/live")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = build_app()
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));

        assert_eq!(response.status(), StatusCode::OK);
        let body = response
            .into_body()
            .collect()
            .await
            .unwrap_or_else(|error| panic!("failed to read response: {error}"))
            .to_bytes();
        assert_eq!(body.as_ref(), r#"{"status":"ok"}"#.as_bytes());
    }

    #[tokio::test]
    async fn version_endpoint_identifies_scaffold() {
        let request = Request::builder()
            .uri("/api/v1/version")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = build_app()
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(response.status(), StatusCode::OK);
    }
}
