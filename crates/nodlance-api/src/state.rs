//! Application state and dependency wiring.

use nodlance_db::{
    PgAgentRepository, PgAlertRepository, PgAuditRepository, PgClusterRepository,
    PgHeartbeatRepository, PgHostRepository, PgObservationRepository, PgUserRepository,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: Option<PgPool>,
    pub jwt_secret: String,
    pub host_repo: PgHostRepository,
    pub agent_repo: PgAgentRepository,
    pub user_repo: PgUserRepository,
    pub audit_repo: PgAuditRepository,
    pub cluster_repo: PgClusterRepository,
    pub heartbeat_repo: PgHeartbeatRepository,
    pub observation_repo: PgObservationRepository,
    pub alert_repo: PgAlertRepository,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        Self {
            host_repo: PgHostRepository::new(pool.clone()),
            agent_repo: PgAgentRepository::new(pool.clone()),
            user_repo: PgUserRepository::new(pool.clone()),
            audit_repo: PgAuditRepository::new(pool.clone()),
            cluster_repo: PgClusterRepository::new(pool.clone()),
            heartbeat_repo: PgHeartbeatRepository::new(pool.clone()),
            observation_repo: PgObservationRepository::new(pool.clone()),
            alert_repo: PgAlertRepository::new(pool.clone()),
            db: Some(pool),
            jwt_secret,
        }
    }

    /// Creates an AppState with lazy/unconnected pool for testing without database.
    pub fn new_test() -> Self {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://localhost/dummy_nodlance")
            .unwrap_or_else(|_| panic!("lazy pool creation failed"));

        Self {
            host_repo: PgHostRepository::new(pool.clone()),
            agent_repo: PgAgentRepository::new(pool.clone()),
            user_repo: PgUserRepository::new(pool.clone()),
            audit_repo: PgAuditRepository::new(pool.clone()),
            cluster_repo: PgClusterRepository::new(pool.clone()),
            heartbeat_repo: PgHeartbeatRepository::new(pool.clone()),
            observation_repo: PgObservationRepository::new(pool.clone()),
            alert_repo: PgAlertRepository::new(pool.clone()),
            db: None,
            jwt_secret: "test-secret-key-that-is-at-least-32-bytes-long".to_string(),
        }
    }
}
