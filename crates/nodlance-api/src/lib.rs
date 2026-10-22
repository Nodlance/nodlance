//! HTTP API for Nodlance control plane.

pub mod auth;
pub mod error;
pub mod routes;
pub mod state;

pub use auth::AuthUser;
pub use error::ApiError;
pub use state::AppState;

use std::path::Path;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::{ServeDir, ServeFile};

/// Builds the API router with all registered routes and state.
pub fn build_app(state: AppState) -> Router {
    let static_path = if let Ok(custom_dir) = std::env::var("NODLANCE_STATIC_DIR") {
        custom_dir
    } else if Path::new("ui/dist/index.html").exists() {
        "ui/dist".to_string()
    } else if Path::new("crates/nodlance-api/static/index.html").exists() {
        "crates/nodlance-api/static".to_string()
    } else if Path::new("static/index.html").exists() {
        "static".to_string()
    } else {
        "ui/dist".to_string()
    };
    let serve_dir =
        ServeDir::new(&static_path).fallback(ServeFile::new(format!("{static_path}/index.html")));
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
        // Load Balancers & Proxies (ProxySQL, PgBouncer, HAProxy, Keepalived)
        .route(
            "/api/v1/load-balancers",
            get(routes::load_balancers::list_load_balancers)
                .post(routes::load_balancers::create_load_balancer),
        )
        .route(
            "/api/v1/load-balancers/{id}",
            get(routes::load_balancers::get_load_balancer)
                .put(routes::load_balancers::update_load_balancer)
                .delete(routes::load_balancers::delete_load_balancer),
        )
        .route(
            "/api/v1/load-balancers/{id}/reload",
            post(routes::load_balancers::reload_load_balancer),
        )
        // Galera Cluster & Garbd Arbitrator
        .route(
            "/api/v1/clusters/{id}/galera/status",
            get(routes::galera::get_galera_status),
        )
        .route(
            "/api/v1/clusters/{id}/galera/arbitrator",
            post(routes::galera::generate_arbitrator_config),
        )
        // Backup Schedules & Continuous PITR Streams
        .route(
            "/api/v1/clusters/{id}/backup-schedules",
            get(routes::backup_schedules::list_cluster_backup_schedules)
                .post(routes::backup_schedules::create_cluster_backup_schedule),
        )
        .route(
            "/api/v1/backup-schedules/{id}",
            get(routes::backup_schedules::get_backup_schedule)
                .put(routes::backup_schedules::update_backup_schedule)
                .delete(routes::backup_schedules::delete_backup_schedule),
        )
        .route(
            "/api/v1/clusters/{id}/pitr/stream",
            get(routes::backup_schedules::get_cluster_pitr_stream)
                .put(routes::backup_schedules::upsert_cluster_pitr_stream),
        )
        // DBA Advisors & Operational Heuristics
        .route(
            "/api/v1/advisors/rules",
            get(routes::advisors::list_advisor_rules)
                .post(routes::advisors::create_or_update_advisor_rule),
        )
        .route(
            "/api/v1/clusters/{id}/advisors/findings",
            get(routes::advisors::list_cluster_advisor_findings),
        )
        .route(
            "/api/v1/clusters/{id}/advisors/scan",
            post(routes::advisors::trigger_cluster_advisor_scan),
        )
        .route(
            "/api/v1/advisors/findings/{id}/resolve",
            post(routes::advisors::resolve_advisor_finding),
        )
        // Query Performance & Slow Query Digest
        .route(
            "/api/v1/clusters/{id}/queries/slow",
            get(routes::advisors::list_cluster_slow_queries),
        )
        .route(
            "/api/v1/clusters/{id}/queries/digest",
            post(routes::advisors::ingest_cluster_query_digests),
        )
        // Centralized Configuration Management
        .route(
            "/api/v1/configs/templates",
            get(routes::config_pki::list_templates)
                .post(routes::config_pki::create_or_update_template),
        )
        .route(
            "/api/v1/configs/templates/{id}",
            get(routes::config_pki::get_template).delete(routes::config_pki::delete_template),
        )
        .route(
            "/api/v1/clusters/{id}/configs/diff",
            post(routes::config_pki::calculate_diff),
        )
        // Built-in mTLS PKI Certificate Authority
        .route(
            "/api/v1/clusters/{id}/pki/certificates",
            get(routes::config_pki::list_certificates),
        )
        .route(
            "/api/v1/clusters/{id}/pki/generate",
            post(routes::config_pki::generate_certificate),
        )
        .route(
            "/api/v1/pki/certificates/{id}/revoke",
            post(routes::config_pki::revoke_certificate),
        )
        .fallback_service(serve_dir)
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
    use uuid::Uuid;

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

    #[tokio::test]
    async fn load_balancers_endpoints_require_authorization() {
        let app = build_app(AppState::new_test());

        let request = Request::builder()
            .uri("/api/v1/load-balancers")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .clone()
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let request = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/load-balancers/{}/reload", Uuid::now_v7()))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn galera_endpoints_require_authorization() {
        let app = build_app(AppState::new_test());

        let request = Request::builder()
            .uri(format!("/api/v1/clusters/{}/galera/status", Uuid::now_v7()))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn backup_schedule_endpoints_require_authorization() {
        let app = build_app(AppState::new_test());

        let request = Request::builder()
            .uri(format!(
                "/api/v1/clusters/{}/backup-schedules",
                Uuid::now_v7()
            ))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .clone()
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let pitr_request = Request::builder()
            .uri(format!("/api/v1/clusters/{}/pitr/stream", Uuid::now_v7()))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let pitr_response = app
            .oneshot(pitr_request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(pitr_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn advisors_endpoints_require_authorization() {
        let app = build_app(AppState::new_test());

        let request = Request::builder()
            .uri("/api/v1/advisors/rules")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .clone()
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let scan_request = Request::builder()
            .uri(format!("/api/v1/clusters/{}/advisors/scan", Uuid::now_v7()))
            .method("POST")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let scan_response = app
            .oneshot(scan_request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(scan_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn config_and_pki_endpoints_require_authorization() {
        let app = build_app(AppState::new_test());

        let template_req = Request::builder()
            .uri("/api/v1/configs/templates")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .clone()
            .oneshot(template_req)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let pki_req = Request::builder()
            .uri(format!(
                "/api/v1/clusters/{}/pki/certificates",
                Uuid::now_v7()
            ))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let pki_resp = app
            .oneshot(pki_req)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));
        assert_eq!(pki_resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn static_ui_root_endpoint_returns_ok() {
        let app = build_app(AppState::new_test());
        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("failed to build request: {error}"));
        let response = app
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("router failed: {error}"));

        assert_eq!(response.status(), StatusCode::OK);
    }
}
