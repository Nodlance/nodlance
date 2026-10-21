//! Application state and dependency wiring.

use nodlance_db::{
    PgAgentRepository, PgAlertRepository, PgAuditRepository, PgBackupRepository,
    PgClusterRepository, PgDiscoveryRepository, PgHeartbeatRepository, PgHostRepository,
    PgJobRepository, PgObservationRepository, PgProvisioningRepository, PgRecoveryRepository,
    PgUserRepository,
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
    pub job_repo: PgJobRepository,
    pub backup_repo: PgBackupRepository,
    pub provisioning_repo: PgProvisioningRepository,
    pub discovery_repo: PgDiscoveryRepository,
    pub recovery_repo: PgRecoveryRepository,
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
            job_repo: PgJobRepository::new(pool.clone()),
            backup_repo: PgBackupRepository::new(pool.clone()),
            provisioning_repo: PgProvisioningRepository::new(pool.clone()),
            discovery_repo: PgDiscoveryRepository::new(pool.clone()),
            recovery_repo: PgRecoveryRepository::new(pool.clone()),
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
            job_repo: PgJobRepository::new(pool.clone()),
            backup_repo: PgBackupRepository::new(pool.clone()),
            provisioning_repo: PgProvisioningRepository::new(pool.clone()),
            discovery_repo: PgDiscoveryRepository::new(pool.clone()),
            recovery_repo: PgRecoveryRepository::new(pool.clone()),
            db: None,
            jwt_secret: "test-secret-key-that-is-at-least-32-bytes-long".to_string(),
        }
    }
}
