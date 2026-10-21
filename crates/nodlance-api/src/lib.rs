//! HTTP API for Nodlance control plane.

pub mod auth;
pub mod error;
pub mod routes;
pub mod state;

pub use auth::AuthUser;
pub use error::ApiError;
pub use state::AppState;

use axum::{
    Router,
    routing::{get, post},
};

/// Builds the API router with all registered routes and state.
pub fn build_app(state: AppState) -> Router {
    Router::new()
        // Health probes
        .route("/health/live", get(routes::health::health_live))
        .route("/health/ready", get(routes::health::health_ready))
        // Version
        .route("/api/v1/version", get(routes::health::version))
        // Authentication
        .route("/api/v1/auth/login", post(routes::auth::login))
        // Users
        .route(
            "/api/v1/users",
            get(routes::users::list_users).post(routes::users::create_user),
        )
        // Hosts
        .route(
            "/api/v1/hosts",
            get(routes::hosts::list_hosts).post(routes::hosts::create_host),
        )
        .route(
            "/api/v1/hosts/{id}",
            get(routes::hosts::get_host)
                .put(routes::hosts::update_host)
                .delete(routes::hosts::decommission_host),
        )
        // Agent Enrollment
        .route(
            "/api/v1/hosts/{id}/enroll",
            post(routes::hosts::enroll_agent),
        )
        .route(
            "/api/v1/hosts/{id}/revoke",
            post(routes::hosts::revoke_agent),
        )
        // Audit log
        .route("/api/v1/audit", get(routes::audit::list_audit_events))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn liveness_endpoint_returns_ok() {
        let app = build_app(AppState::new_test());
        let request = Request::builder()
            .uri("/health/live")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
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
    async fn readiness_endpoint_returns_ready() {
        let app = build_app(AppState::new_test());
        let request = Request::builder()
            .uri("/health/ready")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn version_endpoint_identifies_inventory_stage() {
        let app = build_app(AppState::new_test());
        let request = Request::builder()
            .uri("/api/v1/version")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn protected_endpoints_require_authorization() {
        let app = build_app(AppState::new_test());
        let request = Request::builder()
            .uri("/api/v1/hosts")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
