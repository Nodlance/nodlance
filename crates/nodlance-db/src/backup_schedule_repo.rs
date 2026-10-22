//! PostgreSQL implementation of BackupScheduleRepository.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use nodlance_domain::{
    BackupSchedule, BackupScheduleRepository, CompressionFormat, DomainError, NewBackupSchedule,
    PitrStream, StorageTarget, UpdateBackupSchedule,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgBackupScheduleRepository {
    pool: PgPool,
}

impl PgBackupScheduleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct BackupScheduleRow {
    id: Uuid,
    cluster_id: Uuid,
    name: String,
    cron_expression: String,
    backup_method: String,
    storage_target: String,
    retention_days: i32,
    compression: String,
    encrypt: bool,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<BackupScheduleRow> for BackupSchedule {
    type Error = DomainError;

    fn try_from(row: BackupScheduleRow) -> Result<Self, Self::Error> {
        let storage_target = StorageTarget::from_str(&row.storage_target)
            .map_err(|e| DomainError::Internal(format!("corrupt storage_target in db: {e}")))?;

        let compression = CompressionFormat::from_str(&row.compression)
            .map_err(|e| DomainError::Internal(format!("corrupt compression in db: {e}")))?;

        Ok(Self {
            id: row.id,
            cluster_id: row.cluster_id,
            name: row.name,
            cron_expression: row.cron_expression,
            backup_method: row.backup_method,
            storage_target,
            retention_days: row.retention_days.max(1) as u32,
            compression,
            encrypt: row.encrypt,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(FromRow)]
struct PitrStreamRow {
    id: Uuid,
    cluster_id: Uuid,
    storage_path: String,
    start_time: DateTime<Utc>,
    latest_time: DateTime<Utc>,
    start_lsn_or_gtid: String,
    latest_lsn_or_gtid: String,
    status: String,
    updated_at: DateTime<Utc>,
}

impl From<PitrStreamRow> for PitrStream {
    fn from(row: PitrStreamRow) -> Self {
        Self {
            id: row.id,
            cluster_id: row.cluster_id,
            storage_path: row.storage_path,
            start_time: row.start_time,
            latest_time: row.latest_time,
            start_lsn_or_gtid: row.start_lsn_or_gtid,
            latest_lsn_or_gtid: row.latest_lsn_or_gtid,
            status: row.status,
            updated_at: row.updated_at,
        }
    }
}

impl BackupScheduleRepository for PgBackupScheduleRepository {
    async fn create_schedule(
        &self,
        schedule: &NewBackupSchedule,
    ) -> Result<BackupSchedule, DomainError> {
        schedule.validate()?;

        let id = Uuid::now_v7();
        let target_str = schedule.storage_target.as_str();
        let compression_str = schedule.compression.as_str();
        let retention_days = schedule.retention_days as i32;

        let row = sqlx::query_as::<_, BackupScheduleRow>(
            r#"
            INSERT INTO backup_schedules (
                id, cluster_id, name, cron_expression, backup_method, storage_target,
                retention_days, compression, encrypt, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
            RETURNING id, cluster_id, name, cron_expression, backup_method, storage_target,
                      retention_days, compression, encrypt, is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(schedule.cluster_id)
        .bind(&schedule.name)
        .bind(&schedule.cron_expression)
        .bind(&schedule.backup_method)
        .bind(target_str)
        .bind(retention_days)
        .bind(compression_str)
        .bind(schedule.encrypt)
        .bind(schedule.is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "create backup schedule"))?;

        BackupSchedule::try_from(row)
    }

    async fn get_schedule_by_id(&self, id: Uuid) -> Result<Option<BackupSchedule>, DomainError> {
        let opt = sqlx::query_as::<_, BackupScheduleRow>(
            r#"
            SELECT id, cluster_id, name, cron_expression, backup_method, storage_target,
                   retention_days, compression, encrypt, is_active, created_at, updated_at
            FROM backup_schedules
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get backup schedule by id"))?;

        match opt {
            Some(row) => Ok(Some(BackupSchedule::try_from(row)?)),
            None => Ok(None),
        }
    }

    async fn list_schedules_by_cluster(
        &self,
        cluster_id: Uuid,
    ) -> Result<Vec<BackupSchedule>, DomainError> {
        let rows = sqlx::query_as::<_, BackupScheduleRow>(
            r#"
            SELECT id, cluster_id, name, cron_expression, backup_method, storage_target,
                   retention_days, compression, encrypt, is_active, created_at, updated_at
            FROM backup_schedules
            WHERE cluster_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(cluster_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list backup schedules by cluster"))?;

        rows.into_iter().map(BackupSchedule::try_from).collect()
    }

    async fn update_schedule(
        &self,
        id: Uuid,
        update: &UpdateBackupSchedule,
    ) -> Result<BackupSchedule, DomainError> {
        let retention_days = update.retention_days.map(|d| d as i32);

        let row = sqlx::query_as::<_, BackupScheduleRow>(
            r#"
            UPDATE backup_schedules
            SET name = COALESCE($2, name),
                cron_expression = COALESCE($3, cron_expression),
                retention_days = COALESCE($4, retention_days),
                is_active = COALESCE($5, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, cluster_id, name, cron_expression, backup_method, storage_target,
                      retention_days, compression, encrypt, is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&update.name)
        .bind(&update.cron_expression)
        .bind(retention_days)
        .bind(update.is_active)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "update backup schedule"))?
        .ok_or_else(|| DomainError::NotFound(format!("backup schedule '{id}' not found")))?;

        BackupSchedule::try_from(row)
    }

    async fn delete_schedule(&self, id: Uuid) -> Result<(), DomainError> {
        let res = sqlx::query(
            r#"
            DELETE FROM backup_schedules
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "delete backup schedule"))?;

        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!(
                "backup schedule '{id}' not found"
            )));
        }

        Ok(())
    }

    async fn get_pitr_stream_by_cluster(
        &self,
        cluster_id: Uuid,
    ) -> Result<Option<PitrStream>, DomainError> {
        let opt = sqlx::query_as::<_, PitrStreamRow>(
            r#"
            SELECT id, cluster_id, storage_path, start_time, latest_time,
                   start_lsn_or_gtid, latest_lsn_or_gtid, status, updated_at
            FROM pitr_streams
            WHERE cluster_id = $1
            "#,
        )
        .bind(cluster_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get pitr stream by cluster"))?;

        Ok(opt.map(PitrStream::from))
    }

    async fn upsert_pitr_stream(&self, stream: &PitrStream) -> Result<PitrStream, DomainError> {
        let row = sqlx::query_as::<_, PitrStreamRow>(
            r#"
            INSERT INTO pitr_streams (
                id, cluster_id, storage_path, start_time, latest_time,
                start_lsn_or_gtid, latest_lsn_or_gtid, status, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            ON CONFLICT (cluster_id) DO UPDATE SET
                storage_path = EXCLUDED.storage_path,
                start_time = EXCLUDED.start_time,
                latest_time = EXCLUDED.latest_time,
                start_lsn_or_gtid = EXCLUDED.start_lsn_or_gtid,
                latest_lsn_or_gtid = EXCLUDED.latest_lsn_or_gtid,
                status = EXCLUDED.status,
                updated_at = NOW()
            RETURNING id, cluster_id, storage_path, start_time, latest_time,
                      start_lsn_or_gtid, latest_lsn_or_gtid, status, updated_at
            "#,
        )
        .bind(stream.id)
        .bind(stream.cluster_id)
        .bind(&stream.storage_path)
        .bind(stream.start_time)
        .bind(stream.latest_time)
        .bind(&stream.start_lsn_or_gtid)
        .bind(&stream.latest_lsn_or_gtid)
        .bind(&stream.status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "upsert pitr stream"))?;

        Ok(PitrStream::from(row))
    }
}
