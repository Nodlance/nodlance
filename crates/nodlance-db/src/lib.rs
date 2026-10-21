//! PostgreSQL metadata storage layer for Nodlance.

pub mod agent_repo;
pub mod alert_repo;
pub mod audit_repo;
pub mod backup_repo;
pub mod cluster_repo;
pub mod error;
pub mod heartbeat_repo;
pub mod host_repo;
pub mod job_repo;
pub mod observation_repo;
pub mod user_repo;

pub use agent_repo::PgAgentRepository;
pub use alert_repo::PgAlertRepository;
pub use audit_repo::PgAuditRepository;
pub use backup_repo::PgBackupRepository;
pub use cluster_repo::PgClusterRepository;
pub use heartbeat_repo::PgHeartbeatRepository;
pub use host_repo::PgHostRepository;
pub use job_repo::PgJobRepository;
pub use observation_repo::PgObservationRepository;
pub use user_repo::PgUserRepository;

use nodlance_domain::{DomainError, UserRepository};
use sqlx::{PgPool, postgres::PgPoolOptions};

/// Create a configured PostgreSQL connection pool.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
}

/// Run all embedded database migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Seeds the initial admin user if no admin user exists.
pub async fn seed_admin_if_needed(pool: &PgPool, password_hash: &str) -> Result<(), DomainError> {
    let user_repo = PgUserRepository::new(pool.clone());
    let existing = user_repo.get_by_username("admin").await?;

    if existing.is_none() {
        tracing::info!("seeding initial admin user");
        let new_user = nodlance_domain::NewUser {
            username: "admin".to_string(),
            password: String::new(), // Not stored directly; hash is passed
            display_name: Some("System Administrator".to_string()),
        };
        let user = user_repo.create(&new_user, password_hash).await?;
        user_repo.assign_role(user.id, "admin").await?;
        tracing::info!(user_id = %user.id, "initial admin user created and assigned admin role");
    }

    Ok(())
}
