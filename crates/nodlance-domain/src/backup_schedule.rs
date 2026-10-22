//! Domain models for automated backup scheduling, continuous Point-in-Time Recovery (PITR) streams,
//! and cloud storage offloading (AWS S3, MinIO, GCS, Azure Blob).

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

/// Storage target for durable backup archives and continuous WAL/binlog streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageTarget {
    Local,
    S3,
    Gcs,
    AzureBlob,
    Minio,
}

impl StorageTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::S3 => "s3",
            Self::Gcs => "gcs",
            Self::AzureBlob => "azure_blob",
            Self::Minio => "minio",
        }
    }
}

impl fmt::Display for StorageTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for StorageTarget {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "s3" | "aws_s3" => Ok(Self::S3),
            "gcs" | "google_cloud" => Ok(Self::Gcs),
            "azure" | "azure_blob" => Ok(Self::AzureBlob),
            "minio" => Ok(Self::Minio),
            other => Err(DomainError::Validation(format!(
                "invalid storage target: '{other}'"
            ))),
        }
    }
}

/// Compression format applied to backup artifacts before storage upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionFormat {
    Zstd,
    Lz4,
    Gzip,
    None,
}

impl CompressionFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Zstd => "zstd",
            Self::Lz4 => "lz4",
            Self::Gzip => "gzip",
            Self::None => "none",
        }
    }
}

impl fmt::Display for CompressionFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CompressionFormat {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zstd" => Ok(Self::Zstd),
            "lz4" => Ok(Self::Lz4),
            "gzip" => Ok(Self::Gzip),
            "none" | "" => Ok(Self::None),
            other => Err(DomainError::Validation(format!(
                "invalid compression format: '{other}'"
            ))),
        }
    }
}

/// Scheduled automated backup configuration for a database cluster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub name: String,
    pub cron_expression: String,
    pub backup_method: String,
    pub storage_target: StorageTarget,
    pub retention_days: u32,
    pub compression: CompressionFormat,
    pub encrypt: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BackupSchedule {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() {
            return Err(DomainError::Validation(
                "schedule name cannot be empty".to_string(),
            ));
        }

        // Basic cron validation (must have at least 5 space-delimited tokens)
        let token_count = self.cron_expression.split_whitespace().count();
        if !(5..=6).contains(&token_count) {
            return Err(DomainError::Validation(format!(
                "invalid cron expression '{}': must have 5 or 6 fields",
                self.cron_expression
            )));
        }

        if self.retention_days == 0 {
            return Err(DomainError::Validation(
                "retention_days must be at least 1 day".to_string(),
            ));
        }

        Ok(())
    }
}

/// DTO for creating a new automated backup schedule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewBackupSchedule {
    pub cluster_id: Uuid,
    pub name: String,
    pub cron_expression: String,
    pub backup_method: String,
    pub storage_target: StorageTarget,
    pub retention_days: u32,
    pub compression: CompressionFormat,
    pub encrypt: bool,
    #[serde(default = "default_active")]
    pub is_active: bool,
}

fn default_active() -> bool {
    true
}

impl NewBackupSchedule {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() {
            return Err(DomainError::Validation(
                "schedule name cannot be empty".to_string(),
            ));
        }

        let token_count = self.cron_expression.split_whitespace().count();
        if !(5..=6).contains(&token_count) {
            return Err(DomainError::Validation(format!(
                "invalid cron expression '{}': must have 5 or 6 fields",
                self.cron_expression
            )));
        }

        if self.retention_days == 0 {
            return Err(DomainError::Validation(
                "retention_days must be at least 1 day".to_string(),
            ));
        }

        Ok(())
    }
}

/// DTO for updating an existing backup schedule.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateBackupSchedule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron_expression: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_days: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// Continuous Point-in-Time Recovery (PITR) WAL / Binlog stream catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PitrStream {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub storage_path: String,
    pub start_time: DateTime<Utc>,
    pub latest_time: DateTime<Utc>,
    pub start_lsn_or_gtid: String,
    pub latest_lsn_or_gtid: String,
    pub status: String, // 'active', 'replaying', 'archived'
    pub updated_at: DateTime<Utc>,
}

impl PitrStream {
    /// Calculates the Recovery Point Objective (RPO) lag in seconds relative to current time.
    pub fn calculate_rpo_lag_seconds(&self) -> i64 {
        let diff = Utc::now().signed_duration_since(self.latest_time);
        diff.num_seconds().max(0)
    }

    /// Validates if target recovery timestamp falls within this replay window.
    pub fn contains_timestamp(&self, target: DateTime<Utc>) -> bool {
        target >= self.start_time && target <= self.latest_time
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_target_and_compression_roundtrip() {
        assert_eq!(StorageTarget::from_str("s3").unwrap(), StorageTarget::S3);
        assert_eq!(
            StorageTarget::from_str("minio").unwrap(),
            StorageTarget::Minio
        );
        assert_eq!(
            StorageTarget::from_str("local").unwrap(),
            StorageTarget::Local
        );
        assert!(StorageTarget::from_str("invalid").is_err());

        assert_eq!(
            CompressionFormat::from_str("zstd").unwrap(),
            CompressionFormat::Zstd
        );
        assert_eq!(
            CompressionFormat::from_str("lz4").unwrap(),
            CompressionFormat::Lz4
        );
        assert_eq!(
            CompressionFormat::from_str("gzip").unwrap(),
            CompressionFormat::Gzip
        );
        assert!(CompressionFormat::from_str("bad").is_err());
    }

    #[test]
    fn test_backup_schedule_validation() {
        let valid = NewBackupSchedule {
            cluster_id: Uuid::now_v7(),
            name: "Daily Production Full Backup".to_string(),
            cron_expression: "0 2 * * *".to_string(), // every day at 02:00
            backup_method: "pg_basebackup".to_string(),
            storage_target: StorageTarget::S3,
            retention_days: 30,
            compression: CompressionFormat::Zstd,
            encrypt: true,
            is_active: true,
        };
        assert!(valid.validate().is_ok());

        let invalid_cron = NewBackupSchedule {
            cron_expression: "not a cron".to_string(),
            ..valid.clone()
        };
        assert!(invalid_cron.validate().is_err());

        let zero_retention = NewBackupSchedule {
            retention_days: 0,
            ..valid
        };
        assert!(zero_retention.validate().is_err());
    }

    #[test]
    fn test_pitr_stream_rpo_and_window() {
        let start = Utc::now() - chrono::Duration::hours(24);
        let latest = Utc::now() - chrono::Duration::minutes(5);

        let stream = PitrStream {
            id: Uuid::now_v7(),
            cluster_id: Uuid::now_v7(),
            storage_path: "s3://backups/cluster-1/wal/".to_string(),
            start_time: start,
            latest_time: latest,
            start_lsn_or_gtid: "0/1A0000".to_string(),
            latest_lsn_or_gtid: "0/1F8000".to_string(),
            status: "active".to_string(),
            updated_at: Utc::now(),
        };

        assert!(stream.contains_timestamp(start + chrono::Duration::hours(2)));
        assert!(!stream.contains_timestamp(start - chrono::Duration::hours(1)));
        assert!(stream.calculate_rpo_lag_seconds() >= 290);
    }
}
