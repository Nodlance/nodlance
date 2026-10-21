//! Recovery, High Availability, fencing, and promotion domain models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Monotonically increasing cluster term / epoch tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterEpoch {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub epoch: u64,
    pub promoted_host_id: Option<Uuid>,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

/// Mechanism used to fence / neutralize an unhealthy or old primary host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FencingType {
    PowerStonith,
    NetworkIsolation,
    StorageDetach,
    CustomWebhook,
}

impl FencingType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PowerStonith => "power_stonith",
            Self::NetworkIsolation => "network_isolation",
            Self::StorageDetach => "storage_detach",
            Self::CustomWebhook => "custom_webhook",
        }
    }
}

impl std::str::FromStr for FencingType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "power_stonith" => Ok(Self::PowerStonith),
            "network_isolation" => Ok(Self::NetworkIsolation),
            "storage_detach" => Ok(Self::StorageDetach),
            "custom_webhook" => Ok(Self::CustomWebhook),
            other => Err(format!("unknown fencing type: {other}")),
        }
    }
}

/// Verification status of a fencing operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FencingStatus {
    Pending,
    Executing,
    Verified,
    Failed,
}

impl FencingStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Executing => "executing",
            Self::Verified => "verified",
            Self::Failed => "failed",
        }
    }
}

impl std::str::FromStr for FencingStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "executing" => Ok(Self::Executing),
            "verified" => Ok(Self::Verified),
            "failed" => Ok(Self::Failed),
            other => Err(format!("unknown fencing status: {other}")),
        }
    }
}

/// Executed fencing attempt with verifiable proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FencingExecution {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub target_host_id: Uuid,
    pub fencing_type: FencingType,
    pub status: FencingStatus,
    pub verification_proof: Option<String>,
    pub error_message: Option<String>,
    pub executed_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

/// Status of an operator-approved promotion lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionStatus {
    #[default]
    PendingApproval,
    Approved,
    Executing,
    Promoted,
    Rejected,
    Failed,
}

impl PromotionStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PendingApproval => "pending_approval",
            Self::Approved => "approved",
            Self::Executing => "executing",
            Self::Promoted => "promoted",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
        }
    }
}

impl std::str::FromStr for PromotionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending_approval" => Ok(Self::PendingApproval),
            "approved" => Ok(Self::Approved),
            "executing" => Ok(Self::Executing),
            "promoted" => Ok(Self::Promoted),
            "rejected" => Ok(Self::Rejected),
            "failed" => Ok(Self::Failed),
            other => Err(format!("unknown promotion status: {other}")),
        }
    }
}

/// Candidate replica evaluated for failover promotion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionCandidate {
    pub host_id: Uuid,
    pub role: String,
    pub replication_lag_bytes: u64,
    pub replay_latency_ms: u64,
    pub is_durable: bool,
    pub health: String,
    pub suitability_score: u32,
}

/// Durable promotion request requiring operator approval and safety validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionRequest {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub candidate_host_id: Uuid,
    pub target_epoch: u64,
    pub status: PromotionStatus,
    pub approval_token: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub fencing_execution_id: Option<Uuid>,
    pub safety_gates_passed: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload to request a new promotion evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewPromotionRequest {
    pub cluster_id: Uuid,
    pub candidate_host_id: Uuid,
    pub target_epoch: u64,
}

/// Parameters for updating a promotion request's status and metadata.
#[derive(Debug, Clone, Default)]
pub struct UpdatePromotionParams<'a> {
    pub status: PromotionStatus,
    pub approval_token: Option<&'a str>,
    pub approved_by: Option<Uuid>,
    pub fencing_id: Option<Uuid>,
    pub safety_gates_passed: Option<bool>,
    pub error_message: Option<&'a str>,
}

/// Pre-promotion safety gates evaluation results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionSafetyGates {
    pub quorum_nodes_reachable: u32,
    pub quorum_nodes_total: u32,
    pub quorum_satisfied: bool,
    pub fencing_verified: bool,
    pub max_observed_lag_bytes: u64,
    pub lag_acceptable: bool,
    pub all_passed: bool,
}

/// Scenarios evaluated by the fault injection qualification suite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultInjectionScenario {
    NetworkPartition,
    AsymmetricPartition,
    DelayedStaleCommand,
    FencingFailure,
    AgentCrash,
}

impl FaultInjectionScenario {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NetworkPartition => "network_partition",
            Self::AsymmetricPartition => "asymmetric_partition",
            Self::DelayedStaleCommand => "delayed_stale_command",
            Self::FencingFailure => "fencing_failure",
            Self::AgentCrash => "agent_crash",
        }
    }
}

impl std::str::FromStr for FaultInjectionScenario {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "network_partition" => Ok(Self::NetworkPartition),
            "asymmetric_partition" => Ok(Self::AsymmetricPartition),
            "delayed_stale_command" => Ok(Self::DelayedStaleCommand),
            "fencing_failure" => Ok(Self::FencingFailure),
            "agent_crash" => Ok(Self::AgentCrash),
            other => Err(format!("unknown fault injection scenario: {other}")),
        }
    }
}

/// Qualification report from running fault injection testing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultInjectionResult {
    pub scenario: FaultInjectionScenario,
    pub passed: bool,
    pub split_brain_prevented: bool,
    pub stale_command_rejected: bool,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fencing_type_roundtrip() {
        for ft in &[
            FencingType::PowerStonith,
            FencingType::NetworkIsolation,
            FencingType::StorageDetach,
            FencingType::CustomWebhook,
        ] {
            let s = ft.as_str();
            assert_eq!(s.parse::<FencingType>().expect("parse"), *ft);
        }
    }

    #[test]
    fn test_promotion_status_roundtrip() {
        for ps in &[
            PromotionStatus::PendingApproval,
            PromotionStatus::Approved,
            PromotionStatus::Executing,
            PromotionStatus::Promoted,
            PromotionStatus::Rejected,
            PromotionStatus::Failed,
        ] {
            let s = ps.as_str();
            assert_eq!(s.parse::<PromotionStatus>().expect("parse"), *ps);
        }
    }

    #[test]
    fn test_fault_scenario_roundtrip() {
        for fs in &[
            FaultInjectionScenario::NetworkPartition,
            FaultInjectionScenario::AsymmetricPartition,
            FaultInjectionScenario::DelayedStaleCommand,
            FaultInjectionScenario::FencingFailure,
            FaultInjectionScenario::AgentCrash,
        ] {
            let s = fs.as_str();
            assert_eq!(s.parse::<FaultInjectionScenario>().expect("parse"), *fs);
        }
    }
}
