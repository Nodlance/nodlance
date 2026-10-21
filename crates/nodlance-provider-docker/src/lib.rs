//! Docker container guarded provisioning provider with pinned tags, resource quotas, and rollback.

use nodlance_domain::{DatabaseEngine, DomainError, ProvisioningPlan, ProvisioningStepType};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum DockerProviderError {
    #[error("unsupported engine/version for Docker container: {0}")]
    UnsupportedEngine(String),
    #[error("invalid container specification: {0}")]
    InvalidSpec(String),
    #[error("docker runtime error: {0}")]
    Runtime(String),
}

impl From<DockerProviderError> for DomainError {
    fn from(err: DockerProviderError) -> Self {
        match err {
            DockerProviderError::UnsupportedEngine(msg) => DomainError::Validation(msg),
            DockerProviderError::InvalidSpec(msg) => DomainError::Validation(msg),
            DockerProviderError::Runtime(msg) => DomainError::Internal(msg),
        }
    }
}

/// A guarded, fully specified Docker container configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerContainerSpec {
    pub container_name: String,
    pub image: String,
    pub host_port: u16,
    pub container_port: u16,
    pub volume_name: String,
    pub mount_path: String,
    pub user: String,
    pub nano_cpus: Option<i64>,
    pub memory_bytes: Option<i64>,
    pub env_vars: Vec<(String, String)>,
    pub healthcheck_cmd: Vec<String>,
}

/// Resolves an immutable, pinned Docker image tag for the requested engine and version.
pub fn resolve_docker_image(
    engine: DatabaseEngine,
    version: &str,
) -> Result<String, DockerProviderError> {
    match engine {
        DatabaseEngine::PostgreSql => {
            let major = version
                .split('.')
                .next()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(16);
            if !(13..=18).contains(&major) {
                return Err(DockerProviderError::UnsupportedEngine(format!(
                    "PostgreSQL major version {major} is not supported in Docker provider"
                )));
            }
            Ok(format!("postgres:{major}-alpine"))
        }
        DatabaseEngine::MySql => {
            if version.starts_with("8.0") {
                Ok("mysql:8.0".to_string())
            } else if version.starts_with("8.4") {
                Ok("mysql:8.4".to_string())
            } else {
                Err(DockerProviderError::UnsupportedEngine(format!(
                    "MySQL version {version} is not supported in Docker provider"
                )))
            }
        }
        DatabaseEngine::MariaDb => Ok("mariadb:11.4".to_string()),
    }
}

/// Builds a hardened DockerContainerSpec from a validated ProvisioningPlan.
pub fn build_docker_spec(
    plan: &ProvisioningPlan,
) -> Result<DockerContainerSpec, DockerProviderError> {
    let image = if let Some(custom_image) = &plan.config.container_image {
        custom_image.clone()
    } else {
        resolve_docker_image(plan.engine, &plan.engine_version)?
    };

    let (default_port, mount_path, user, healthcheck) = match plan.engine {
        DatabaseEngine::PostgreSql => (
            5432,
            "/var/lib/postgresql/data".to_string(),
            "999:999".to_string(),
            vec![
                "CMD-SHELL".to_string(),
                "pg_isready -U postgres".to_string(),
            ],
        ),
        DatabaseEngine::MySql | DatabaseEngine::MariaDb => (
            3306,
            "/var/lib/mysql".to_string(),
            "999:999".to_string(),
            vec![
                "CMD-SHELL".to_string(),
                "mysqladmin ping -h localhost".to_string(),
            ],
        ),
    };

    let host_port = plan.config.port.unwrap_or(default_port);
    let container_name = format!("nodlance-{}-{}", plan.engine.as_str(), plan.id);
    let volume_name = format!("nodlance_vol_{}", plan.id);

    let nano_cpus = plan
        .config
        .cpu_limit_cores
        .map(|c| (c * 1_000_000_000.0) as i64);
    let memory_bytes = plan
        .config
        .memory_limit_mb
        .map(|m| (m * 1024 * 1024) as i64);

    let mut env_vars = vec![("NODLANCE_MANAGED".to_string(), "true".to_string())];
    if plan.engine == DatabaseEngine::PostgreSql {
        env_vars.push((
            "PGDATA".to_string(),
            "/var/lib/postgresql/data/pgdata".to_string(),
        ));
    }

    Ok(DockerContainerSpec {
        container_name,
        image,
        host_port,
        container_port: default_port,
        volume_name,
        mount_path,
        user,
        nano_cpus,
        memory_bytes,
        env_vars,
        healthcheck_cmd: healthcheck,
    })
}

/// Rollback actions for Docker containers and associated volumes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockerRollbackAction {
    StopContainer(String),
    RemoveContainer(String),
    RemoveVolume(String),
}

pub struct DockerRollbackPlanner;

impl DockerRollbackPlanner {
    pub fn plan_rollback(
        plan: &ProvisioningPlan,
        completed_steps: &[ProvisioningStepType],
    ) -> Vec<DockerRollbackAction> {
        let mut actions = Vec::new();
        let container_name = format!("nodlance-{}-{}", plan.engine.as_str(), plan.id);
        let volume_name = format!("nodlance_vol_{}", plan.id);

        for step in completed_steps.iter().rev() {
            match step {
                ProvisioningStepType::StartService | ProvisioningStepType::VerifyReadiness => {
                    actions.push(DockerRollbackAction::StopContainer(container_name.clone()));
                }
                ProvisioningStepType::InitializeDatabase
                | ProvisioningStepType::RenderConfiguration => {
                    actions.push(DockerRollbackAction::RemoveContainer(
                        container_name.clone(),
                    ));
                }
                ProvisioningStepType::PrepareStorage => {
                    actions.push(DockerRollbackAction::RemoveVolume(volume_name.clone()));
                }
                ProvisioningStepType::PreflightCheck => {}
            }
        }

        // Deduplicate consecutive actions if any
        actions.dedup();
        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nodlance_domain::{ProviderType, ProvisioningConfig, ProvisioningStatus};
    use uuid::Uuid;

    #[test]
    fn test_resolve_pinned_image() {
        assert_eq!(
            resolve_docker_image(DatabaseEngine::PostgreSql, "16.2")
                .expect("should resolve postgres 16"),
            "postgres:16-alpine"
        );
        assert_eq!(
            resolve_docker_image(DatabaseEngine::MySql, "8.0.36")
                .expect("should resolve mysql 8.0"),
            "mysql:8.0"
        );
        assert!(resolve_docker_image(DatabaseEngine::PostgreSql, "10.0").is_err());
    }

    #[test]
    fn test_build_docker_spec() {
        let plan = ProvisioningPlan {
            id: Uuid::now_v7(),
            cluster_id: Uuid::now_v7(),
            host_id: None,
            provider_type: ProviderType::Docker,
            engine: DatabaseEngine::PostgreSql,
            engine_version: "16.0".to_string(),
            config: ProvisioningConfig {
                port: Some(5432),
                data_directory: None,
                max_connections: Some(100),
                shared_buffers_mb: Some(256),
                memory_limit_mb: Some(2048),
                cpu_limit_cores: Some(1.5),
                container_image: None,
                namespace: None,
                extra_params: None,
            },
            status: ProvisioningStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let spec = build_docker_spec(&plan).expect("build spec");
        assert_eq!(spec.image, "postgres:16-alpine");
        assert_eq!(spec.host_port, 5432);
        assert_eq!(spec.memory_bytes, Some(2048 * 1024 * 1024));
        assert_eq!(spec.nano_cpus, Some(1_500_000_000));
        assert_eq!(spec.user, "999:999");
    }

    #[test]
    fn test_docker_rollback_ordering() {
        let plan = ProvisioningPlan {
            id: Uuid::now_v7(),
            cluster_id: Uuid::now_v7(),
            host_id: None,
            provider_type: ProviderType::Docker,
            engine: DatabaseEngine::PostgreSql,
            engine_version: "16.0".to_string(),
            config: ProvisioningConfig::default(),
            status: ProvisioningStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let steps = [
            ProvisioningStepType::PreflightCheck,
            ProvisioningStepType::PrepareStorage,
            ProvisioningStepType::InitializeDatabase,
            ProvisioningStepType::StartService,
        ];

        let rollback = DockerRollbackPlanner::plan_rollback(&plan, &steps);
        assert_eq!(rollback.len(), 3);
        assert!(matches!(
            rollback[0],
            DockerRollbackAction::StopContainer(_)
        ));
        assert!(matches!(
            rollback[1],
            DockerRollbackAction::RemoveContainer(_)
        ));
        assert!(matches!(rollback[2], DockerRollbackAction::RemoveVolume(_)));
    }
}
