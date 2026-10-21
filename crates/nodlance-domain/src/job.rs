//! Durable job state machine and execution attempt models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::JobStatus;

/// Categorized type of durable background operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    BackupCluster,
    RestoreDrill,
    Reconciliation,
    HealthCheck,
}

impl JobType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BackupCluster => "backup_cluster",
            Self::RestoreDrill => "restore_drill",
            Self::Reconciliation => "reconciliation",
            Self::HealthCheck => "health_check",
        }
    }
}

impl std::str::FromStr for JobType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "backup_cluster" => Ok(Self::BackupCluster),
            "restore_drill" => Ok(Self::RestoreDrill),
            "reconciliation" => Ok(Self::Reconciliation),
            "health_check" => Ok(Self::HealthCheck),
            other => Err(format!("unknown job type: {other}")),
        }
    }
}

/// A durable, leased operation tracked in metadata storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: JobType,
    pub status: JobStatus,
    pub idempotency_key: String,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input to enqueue a new durable job.
#[derive(Debug, Clone, Deserialize)]
pub struct NewJob {
    pub job_type: JobType,
    pub idempotency_key: String,
    pub payload: serde_json::Value,
    pub max_attempts: Option<i32>,
}

/// An execution attempt of a durable job by a worker process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAttempt {
    pub id: Uuid,
    pub job_id: Uuid,
    pub attempt_number: i32,
    pub status: JobStatus,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_type_roundtrip() {
        let types = [
            JobType::BackupCluster,
            JobType::RestoreDrill,
            JobType::Reconciliation,
            JobType::HealthCheck,
        ];
        for t in types {
            let s = t.as_str();
            let parsed: JobType = s.parse().expect("should parse");
            assert_eq!(parsed, t);
        }
    }

    #[test]
    fn test_job_serialization() {
        let job = Job {
            id: Uuid::now_v7(),
            job_type: JobType::BackupCluster,
            status: JobStatus::Queued,
            idempotency_key: "backup-key-1".to_string(),
            payload: serde_json::json!({"cluster_id": "test"}),
            result: None,
            error_message: None,
            lease_owner: None,
            lease_expires_at: None,
            attempt_count: 0,
            max_attempts: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&job).expect("serialize");
        let deserialized: Job = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, job.id);
        assert_eq!(deserialized.job_type, JobType::BackupCluster);
    }
}
