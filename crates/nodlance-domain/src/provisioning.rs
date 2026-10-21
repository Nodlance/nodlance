//! Guarded provisioning plans, steps, compensation rollback, and discovered workloads.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::DatabaseEngine;

/// The target infrastructure provider hosting database nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    LinuxVm,
    Docker,
    Kubernetes,
}

impl ProviderType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LinuxVm => "linux_vm",
            Self::Docker => "docker",
            Self::Kubernetes => "kubernetes",
        }
    }
}

impl std::str::FromStr for ProviderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "linux_vm" => Ok(Self::LinuxVm),
            "docker" => Ok(Self::Docker),
            "kubernetes" => Ok(Self::Kubernetes),
            other => Err(format!("unknown provider type: {other}")),
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Overall execution status of a database provisioning plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisioningStatus {
    Draft,
    Pending,
    Running,
    Succeeded,
    Failed,
    RolledBack,
}

impl ProvisioningStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::RolledBack => "rolled_back",
        }
    }
}

impl std::str::FromStr for ProvisioningStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Self::Draft),
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "rolled_back" => Ok(Self::RolledBack),
            other => Err(format!("unknown provisioning status: {other}")),
        }
    }
}

/// Specific ordered step in a guarded provisioning workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisioningStepType {
    PreflightCheck,
    RenderConfiguration,
    PrepareStorage,
    InitializeDatabase,
    StartService,
    VerifyReadiness,
}

impl ProvisioningStepType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreflightCheck => "preflight_check",
            Self::RenderConfiguration => "render_configuration",
            Self::PrepareStorage => "prepare_storage",
            Self::InitializeDatabase => "initialize_database",
            Self::StartService => "start_service",
            Self::VerifyReadiness => "verify_readiness",
        }
    }
}

impl std::str::FromStr for ProvisioningStepType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "preflight_check" => Ok(Self::PreflightCheck),
            "render_configuration" => Ok(Self::RenderConfiguration),
            "prepare_storage" => Ok(Self::PrepareStorage),
            "initialize_database" => Ok(Self::InitializeDatabase),
            "start_service" => Ok(Self::StartService),
            "verify_readiness" => Ok(Self::VerifyReadiness),
            other => Err(format!("unknown step type: {other}")),
        }
    }
}

/// Status of an individual provisioning step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    RolledBack,
}

impl StepStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::RolledBack => "rolled_back",
        }
    }
}

impl std::str::FromStr for StepStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "rolled_back" => Ok(Self::RolledBack),
            other => Err(format!("unknown step status: {other}")),
        }
    }
}

/// Typed configuration parameters for database provisioning.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvisioningConfig {
    pub port: Option<u16>,
    pub data_directory: Option<String>,
    pub max_connections: Option<u32>,
    pub shared_buffers_mb: Option<u64>,
    pub memory_limit_mb: Option<u64>,
    pub cpu_limit_cores: Option<f64>,
    pub container_image: Option<String>,
    pub namespace: Option<String>,
    pub extra_params: Option<serde_json::Value>,
}

/// A guarded provisioning specification tracked in the control plane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningPlan {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub host_id: Option<Uuid>,
    pub provider_type: ProviderType,
    pub engine: DatabaseEngine,
    pub engine_version: String,
    pub config: ProvisioningConfig,
    pub status: ProvisioningStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input to create a new provisioning plan.
#[derive(Debug, Clone, Deserialize)]
pub struct NewProvisioningPlan {
    pub cluster_id: Uuid,
    pub host_id: Option<Uuid>,
    pub provider_type: ProviderType,
    pub engine: DatabaseEngine,
    pub engine_version: String,
    pub config: ProvisioningConfig,
}

/// An immutable or tracked execution record of a single step within a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningStep {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub step_index: i32,
    pub step_type: ProvisioningStepType,
    pub status: StepStatus,
    pub error_details: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// An externally discovered database workload (e.g. from Kubernetes namespace scan or Docker daemon).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredWorkload {
    pub id: Uuid,
    pub provider_type: ProviderType,
    pub namespace: Option<String>,
    pub workload_name: String,
    pub engine: DatabaseEngine,
    pub detected_version: Option<String>,
    pub endpoints: Vec<String>,
    pub status: String,
    pub details: Option<serde_json::Value>,
    pub last_seen_at: DateTime<Utc>,
}

/// Input to record or upsert a discovered workload.
#[derive(Debug, Clone, Deserialize)]
pub struct NewDiscoveredWorkload {
    pub provider_type: ProviderType,
    pub namespace: Option<String>,
    pub workload_name: String,
    pub engine: DatabaseEngine,
    pub detected_version: Option<String>,
    pub endpoints: Vec<String>,
    pub status: String,
    pub details: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_roundtrip() {
        for p in [
            ProviderType::LinuxVm,
            ProviderType::Docker,
            ProviderType::Kubernetes,
        ] {
            let s = p.as_str();
            let parsed: ProviderType = s.parse().expect("should parse");
            assert_eq!(parsed, p);
        }
    }

    #[test]
    fn test_provisioning_status_roundtrip() {
        for s in [
            ProvisioningStatus::Draft,
            ProvisioningStatus::Pending,
            ProvisioningStatus::Running,
            ProvisioningStatus::Succeeded,
            ProvisioningStatus::Failed,
            ProvisioningStatus::RolledBack,
        ] {
            let str_val = s.as_str();
            let parsed: ProvisioningStatus = str_val.parse().expect("should parse");
            assert_eq!(parsed, s);
        }
    }

    #[test]
    fn test_provisioning_step_type_roundtrip() {
        for step in [
            ProvisioningStepType::PreflightCheck,
            ProvisioningStepType::RenderConfiguration,
            ProvisioningStepType::PrepareStorage,
            ProvisioningStepType::InitializeDatabase,
            ProvisioningStepType::StartService,
            ProvisioningStepType::VerifyReadiness,
        ] {
            let str_val = step.as_str();
            let parsed: ProvisioningStepType = str_val.parse().expect("should parse");
            assert_eq!(parsed, step);
        }
    }

    #[test]
    fn test_provisioning_plan_serialization() {
        let plan = ProvisioningPlan {
            id: Uuid::now_v7(),
            cluster_id: Uuid::now_v7(),
            host_id: Some(Uuid::now_v7()),
            provider_type: ProviderType::LinuxVm,
            engine: DatabaseEngine::PostgreSql,
            engine_version: "16.1".to_string(),
            config: ProvisioningConfig {
                port: Some(5432),
                data_directory: Some("/var/lib/postgresql/16/main".to_string()),
                max_connections: Some(100),
                shared_buffers_mb: Some(256),
                memory_limit_mb: Some(2048),
                cpu_limit_cores: Some(2.0),
                container_image: None,
                namespace: None,
                extra_params: None,
            },
            status: ProvisioningStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&plan).expect("serialize");
        let deserialized: ProvisioningPlan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, plan.id);
        assert_eq!(deserialized.provider_type, ProviderType::LinuxVm);
        assert_eq!(deserialized.engine, DatabaseEngine::PostgreSql);
    }
}
