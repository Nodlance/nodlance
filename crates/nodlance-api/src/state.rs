use nodlance_advisor::AdvisorEvaluator;
use nodlance_db::{
    PgAdvisorRepository, PgAgentRepository, PgAlertRepository, PgAuditRepository,
    PgBackupRepository, PgBackupScheduleRepository, PgClusterRepository, PgConfigurationRepository,
    PgDiscoveryRepository, PgHeartbeatRepository, PgHostRepository, PgJobRepository,
    PgLoadBalancerRepository, PgObservationRepository, PgPkiRepository, PgProvisioningRepository,
    PgRecoveryRepository, PgUserRepository,
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
    pub backup_schedule_repo: PgBackupScheduleRepository,
    pub provisioning_repo: PgProvisioningRepository,
    pub discovery_repo: PgDiscoveryRepository,
    pub recovery_repo: PgRecoveryRepository,
    pub load_balancer_repo: PgLoadBalancerRepository,
    pub advisor_repo: PgAdvisorRepository,
    pub advisor_evaluator: AdvisorEvaluator,
    pub config_repo: PgConfigurationRepository,
    pub pki_repo: PgPkiRepository,
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
            backup_schedule_repo: PgBackupScheduleRepository::new(pool.clone()),
            provisioning_repo: PgProvisioningRepository::new(pool.clone()),
            discovery_repo: PgDiscoveryRepository::new(pool.clone()),
            recovery_repo: PgRecoveryRepository::new(pool.clone()),
            load_balancer_repo: PgLoadBalancerRepository::new(pool.clone()),
            advisor_repo: PgAdvisorRepository::new(pool.clone()),
            advisor_evaluator: AdvisorEvaluator::new(),
            config_repo: PgConfigurationRepository::new(pool.clone()),
            pki_repo: PgPkiRepository::new(pool.clone()),
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
            backup_schedule_repo: PgBackupScheduleRepository::new(pool.clone()),
            provisioning_repo: PgProvisioningRepository::new(pool.clone()),
            discovery_repo: PgDiscoveryRepository::new(pool.clone()),
            recovery_repo: PgRecoveryRepository::new(pool.clone()),
            load_balancer_repo: PgLoadBalancerRepository::new(pool.clone()),
            advisor_repo: PgAdvisorRepository::new(pool.clone()),
            advisor_evaluator: AdvisorEvaluator::new(),
            config_repo: PgConfigurationRepository::new(pool.clone()),
            pki_repo: PgPkiRepository::new(pool.clone()),
            db: None,
            jwt_secret: "test-secret-key-that-is-at-least-32-bytes-long".to_string(),
        }
    }
}
