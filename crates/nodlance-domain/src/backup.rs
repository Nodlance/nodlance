//! Backup artifact metadata, checksums, retention, and restore drills.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::DatabaseEngine;

/// Type of backup snapshot taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupType {
    Full,
    Incremental,
    Differential,
}

impl BackupType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Incremental => "incremental",
            Self::Differential => "differential",
        }
    }
}

impl std::str::FromStr for BackupType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "full" => Ok(Self::Full),
            "incremental" => Ok(Self::Incremental),
            "differential" => Ok(Self::Differential),
            other => Err(format!("unknown backup type: {other}")),
        }
    }
}

/// Integrity and completion status of a backup artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupStatus {
    Pending,
    Completed,
    Corrupt,
    Failed,
}

impl BackupStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Corrupt => "corrupt",
            Self::Failed => "failed",
        }
    }
}

impl std::str::FromStr for BackupStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "completed" => Ok(Self::Completed),
            "corrupt" => Ok(Self::Corrupt),
            "failed" => Ok(Self::Failed),
            other => Err(format!("unknown backup status: {other}")),
        }
    }
}

/// An immutable record of a generated database backup artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupArtifact {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub node_id: Uuid,
    pub job_id: Uuid,
    pub engine: DatabaseEngine,
    pub engine_version: String,
    pub backup_type: BackupType,
    pub storage_location: String,
    pub size_bytes: u64,
    pub checksum_sha256: String,
    pub status: BackupStatus,
    pub retention_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Input to register a completed or pending backup artifact.
#[derive(Debug, Clone, Deserialize)]
pub struct NewBackupArtifact {
    pub cluster_id: Uuid,
    pub node_id: Uuid,
    pub job_id: Uuid,
    pub engine: DatabaseEngine,
    pub engine_version: String,
    pub backup_type: BackupType,
    pub storage_location: String,
    pub size_bytes: u64,
    pub checksum_sha256: String,
    pub retention_until: Option<DateTime<Utc>>,
}

/// A validation drill report verifying the restoreability and checksum of a backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreValidationDrill {
    pub id: Uuid,
    pub backup_id: Uuid,
    pub verified_at: DateTime<Utc>,
    pub is_valid: bool,
    pub verification_report: serde_json::Value,
}

/// Cluster retention policy specifying how long backups must be kept.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub retention_days: i32,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_type_roundtrip() {
        for b_type in [
            BackupType::Full,
            BackupType::Incremental,
            BackupType::Differential,
        ] {
            let s = b_type.as_str();
            let parsed: BackupType = s.parse().expect("should parse");
            assert_eq!(parsed, b_type);
        }
    }

    #[test]
    fn test_backup_status_roundtrip() {
        for status in [
            BackupStatus::Pending,
            BackupStatus::Completed,
            BackupStatus::Corrupt,
            BackupStatus::Failed,
        ] {
            let s = status.as_str();
            let parsed: BackupStatus = s.parse().expect("should parse");
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_backup_artifact_serialization() {
        let artifact = BackupArtifact {
            id: Uuid::now_v7(),
            cluster_id: Uuid::now_v7(),
            node_id: Uuid::now_v7(),
            job_id: Uuid::now_v7(),
            engine: DatabaseEngine::PostgreSql,
            engine_version: "16.1".to_string(),
            backup_type: BackupType::Full,
            storage_location: "/var/backups/pg_1.tar.gz".to_string(),
            size_bytes: 1048576,
            checksum_sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                .to_string(),
            status: BackupStatus::Completed,
            retention_until: None,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&artifact).expect("serialize");
        let deserialized: BackupArtifact = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, artifact.id);
        assert_eq!(deserialized.engine, DatabaseEngine::PostgreSql);
        assert_eq!(deserialized.status, BackupStatus::Completed);
    }
}
