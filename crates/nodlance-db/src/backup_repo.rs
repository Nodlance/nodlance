//! PostgreSQL implementation of BackupRepository.

use chrono::{DateTime, Utc};
use nodlance_domain::{
    BackupArtifact, BackupRepository, BackupStatus, BackupType, DatabaseEngine, DomainError,
    NewBackupArtifact, RestoreValidationDrill,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgBackupRepository {
    pool: PgPool,
}

impl PgBackupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct BackupRow {
    id: Uuid,
    cluster_id: Uuid,
    node_id: Uuid,
    job_id: Uuid,
    engine: String,
    engine_version: String,
    backup_type: String,
    storage_location: String,
    size_bytes: i64,
    checksum_sha256: String,
    status: String,
    retention_until: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl TryFrom<BackupRow> for BackupArtifact {
    type Error = DomainError;

    fn try_from(row: BackupRow) -> Result<Self, Self::Error> {
        let engine = match row.engine.as_str() {
            "postgresql" => DatabaseEngine::PostgreSql,
            "mysql" => DatabaseEngine::MySql,
            "mariadb" => DatabaseEngine::MariaDb,
            other => {
                return Err(DomainError::Internal(format!(
                    "unknown database engine in db: {other}"
                )));
            }
        };

        let backup_type = row
            .backup_type
            .parse::<BackupType>()
            .map_err(|e| DomainError::Internal(format!("invalid backup type in db: {e}")))?;

        let status = row
            .status
            .parse::<BackupStatus>()
            .map_err(|e| DomainError::Internal(format!("invalid backup status in db: {e}")))?;

        Ok(Self {
            id: row.id,
            cluster_id: row.cluster_id,
            node_id: row.node_id,
            job_id: row.job_id,
            engine,
            engine_version: row.engine_version,
            backup_type,
            storage_location: row.storage_location,
            size_bytes: row.size_bytes as u64,
            checksum_sha256: row.checksum_sha256,
            status,
            retention_until: row.retention_until,
            created_at: row.created_at,
        })
    }
}

#[derive(FromRow)]
struct DrillRow {
    id: Uuid,
    backup_id: Uuid,
    verified_at: DateTime<Utc>,
    is_valid: bool,
    verification_report: serde_json::Value,
}

impl From<DrillRow> for RestoreValidationDrill {
    fn from(row: DrillRow) -> Self {
        Self {
            id: row.id,
            backup_id: row.backup_id,
            verified_at: row.verified_at,
            is_valid: row.is_valid,
            verification_report: row.verification_report,
        }
    }
}

impl BackupRepository for PgBackupRepository {
    async fn record_backup(
        &self,
        artifact: &NewBackupArtifact,
    ) -> Result<BackupArtifact, DomainError> {
        let id = Uuid::now_v7();

        let row = sqlx::query_as::<_, BackupRow>(
            r#"
            INSERT INTO backup_artifacts (id, cluster_id, node_id, job_id, engine, engine_version, backup_type, storage_location, size_bytes, checksum_sha256, status, retention_until, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'completed', $11, now())
            RETURNING id, cluster_id, node_id, job_id, engine, engine_version, backup_type, storage_location, size_bytes, checksum_sha256, status, retention_until, created_at
            "#,
        )
        .bind(id)
        .bind(artifact.cluster_id)
        .bind(artifact.node_id)
        .bind(artifact.job_id)
        .bind(artifact.engine.as_str())
        .bind(&artifact.engine_version)
        .bind(artifact.backup_type.as_str())
        .bind(&artifact.storage_location)
        .bind(artifact.size_bytes as i64)
        .bind(&artifact.checksum_sha256)
        .bind(artifact.retention_until)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "record backup artifact"))?;

        row.try_into()
    }

    async fn get_backup_by_id(&self, id: Uuid) -> Result<Option<BackupArtifact>, DomainError> {
        let row = sqlx::query_as::<_, BackupRow>(
            r#"
            SELECT id, cluster_id, node_id, job_id, engine, engine_version, backup_type, storage_location, size_bytes, checksum_sha256, status, retention_until, created_at
            FROM backup_artifacts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get backup artifact by id"))?;

        row.map(BackupArtifact::try_from).transpose()
    }

    async fn list_backups_by_cluster(
        &self,
        cluster_id: Uuid,
    ) -> Result<Vec<BackupArtifact>, DomainError> {
        let rows = sqlx::query_as::<_, BackupRow>(
            r#"
            SELECT id, cluster_id, node_id, job_id, engine, engine_version, backup_type, storage_location, size_bytes, checksum_sha256, status, retention_until, created_at
            FROM backup_artifacts
            WHERE cluster_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(cluster_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list backups by cluster"))?;

        rows.into_iter().map(BackupArtifact::try_from).collect()
    }

    async fn record_restore_drill(
        &self,
        backup_id: Uuid,
        is_valid: bool,
        report: serde_json::Value,
    ) -> Result<RestoreValidationDrill, DomainError> {
        let id = Uuid::now_v7();

        let row = sqlx::query_as::<_, DrillRow>(
            r#"
            INSERT INTO restore_validation_drills (id, backup_id, verified_at, is_valid, verification_report)
            VALUES ($1, $2, now(), $3, $4)
            RETURNING id, backup_id, verified_at, is_valid, verification_report
            "#,
        )
        .bind(id)
        .bind(backup_id)
        .bind(is_valid)
        .bind(report)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "record restore validation drill"))?;

        // If corrupt/invalid, update backup status
        if !is_valid {
            let _ = sqlx::query(
                r#"
                UPDATE backup_artifacts
                SET status = 'corrupt'
                WHERE id = $1
                "#,
            )
            .bind(backup_id)
            .execute(&self.pool)
            .await;
        }

        Ok(row.into())
    }

    async fn get_latest_drill(
        &self,
        backup_id: Uuid,
    ) -> Result<Option<RestoreValidationDrill>, DomainError> {
        let row = sqlx::query_as::<_, DrillRow>(
            r#"
            SELECT id, backup_id, verified_at, is_valid, verification_report
            FROM restore_validation_drills
            WHERE backup_id = $1
            ORDER BY verified_at DESC
            LIMIT 1
            "#,
        )
        .bind(backup_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get latest restore drill"))?;

        Ok(row.map(Into::into))
    }

    async fn cleanup_expired_backups(&self) -> Result<u64, DomainError> {
        let result = sqlx::query(
            r#"
            DELETE FROM backup_artifacts
            WHERE retention_until IS NOT NULL AND retention_until < now()
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "cleanup expired backups"))?;

        Ok(result.rows_affected())
    }
}
