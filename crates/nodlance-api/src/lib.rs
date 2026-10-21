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
        .route(
            "/api/v1/hosts/{id}/metrics",
            get(routes::hosts::get_host_metrics),
        )
        // Agent Enrollment & Heartbeats
        .route(
            "/api/v1/hosts/{id}/enroll",
            post(routes::hosts::enroll_agent),
        )
        .route(
            "/api/v1/hosts/{id}/revoke",
            post(routes::hosts::revoke_agent),
        )
        .route(
            "/api/v1/agent/heartbeat",
            post(routes::agent::receive_heartbeat),
        )
        // Clusters & Topology
        .route(
            "/api/v1/clusters",
            get(routes::clusters::list_clusters).post(routes::clusters::create_cluster),
        )
        .route("/api/v1/clusters/{id}", get(routes::clusters::get_cluster))
        .route(
            "/api/v1/clusters/{id}/nodes",
            post(routes::clusters::add_node_to_cluster),
        )
        .route(
            "/api/v1/clusters/{id}/topology",
            get(routes::clusters::get_cluster_topology),
        )
        // Alerts & Maintenance
        .route(
            "/api/v1/alerts/rules",
            get(routes::alerts::list_alert_rules),
        )
        .route("/api/v1/alerts", get(routes::alerts::list_active_alerts))
        .route(
            "/api/v1/alerts/{id}/acknowledge",
            post(routes::alerts::acknowledge_alert),
        )
        .route(
            "/api/v1/maintenance-windows",
            post(routes::alerts::create_maintenance_window),
        )
        // Audit log
        .route("/api/v1/audit", get(routes::audit::list_audit_events))
        // Backups & Restores
        .route(
            "/api/v1/clusters/{id}/backups",
            post(routes::backups::trigger_cluster_backup)
                .get(routes::backups::list_cluster_backups),
        )
        .route(
            "/api/v1/backups/{id}",
            get(routes::backups::get_backup_details),
        )
        .route(
            "/api/v1/backups/{id}/verify",
            post(routes::backups::verify_backup),
        )
        .route(
            "/api/v1/backups/{id}/drills/latest",
            get(routes::backups::get_latest_drill),
        )
        // Durable Jobs
        .route("/api/v1/jobs", get(routes::jobs::list_jobs))
        .route("/api/v1/jobs/{id}", get(routes::jobs::get_job))
        // Provisioning Plans
        .route(
            "/api/v1/provisioning/plans",
            get(routes::provisioning::list_plans).post(routes::provisioning::create_plan),
        )
        .route(
            "/api/v1/provisioning/plans/{id}",
            get(routes::provisioning::get_plan),
        )
        .route(
            "/api/v1/provisioning/plans/{id}/steps",
            get(routes::provisioning::list_plan_steps),
        )
        .route(
            "/api/v1/provisioning/plans/{id}/execute",
            post(routes::provisioning::execute_plan),
        )
        .route(
            "/api/v1/provisioning/plans/{id}/rollback",
            post(routes::provisioning::rollback_plan),
        )
        // Workload Discovery
        .route(
            "/api/v1/discovery/scan",
            post(routes::discovery::trigger_scan),
        )
        .route(
            "/api/v1/discovery/workloads",
            get(routes::discovery::list_discovered_workloads),
        )
        .route(
            "/api/v1/discovery/workloads/{id}",
            get(routes::discovery::get_discovered_workload),
        )
        // Recovery, Promotion & Fencing
        .route(
            "/api/v1/clusters/{id}/recovery/candidates",
            get(routes::recovery::list_candidates),
        )
        .route(
            "/api/v1/clusters/{id}/recovery/promote",
            post(routes::recovery::request_promotion),
        )
        .route(
            "/api/v1/clusters/{id}/recovery/promote/{req_id}/approve",
            post(routes::recovery::approve_promotion),
        )
        .route(
            "/api/v1/clusters/{id}/recovery/fence",
            post(routes::recovery::fence_node),
        )
        .route(
            "/api/v1/clusters/{id}/recovery/epoch",
            get(routes::recovery::get_cluster_epoch),
        )
        .route(
            "/api/v1/recovery/fault-injection",
            post(routes::recovery::run_fault_injection),
        )
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

    #[tokio::test]
    async fn clusters_endpoint_requires_authorization() {
        let app = build_app(AppState::new_test());
        let request = Request::builder()
            .uri("/api/v1/clusters")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn jobs_endpoint_requires_authorization() {
        let app = build_app(AppState::new_test());
        let request = Request::builder()
            .uri("/api/v1/jobs")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn backups_endpoint_requires_authorization() {
        let app = build_app(AppState::new_test());
        let cluster_id = uuid::Uuid::now_v7();
        let request = Request::builder()
            .uri(format!("/api/v1/clusters/{cluster_id}/backups"))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn provisioning_plans_endpoint_requires_authorization() {
        let app = build_app(AppState::new_test());
        let request = Request::builder()
            .uri("/api/v1/provisioning/plans")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn discovery_workloads_endpoint_requires_authorization() {
        let app = build_app(AppState::new_test());
        let request = Request::builder()
            .uri("/api/v1/discovery/workloads")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn recovery_endpoints_require_authorization() {
        let app = build_app(AppState::new_test());
        let cluster_id = uuid::Uuid::now_v7();

        // 1. candidates endpoint
        let request = Request::builder()
            .uri(format!("/api/v1/clusters/{cluster_id}/recovery/candidates"))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .clone()
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // 2. epoch endpoint
        let request = Request::builder()
            .uri(format!("/api/v1/clusters/{cluster_id}/recovery/epoch"))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .clone()
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // 3. fault injection endpoint
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/recovery/fault-injection")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
