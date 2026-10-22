//! Backup schedule and PITR continuous stream route handlers.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use nodlance_domain::{
    AuditRepository, BackupSchedule, BackupScheduleRepository, ClusterRepository,
    CompressionFormat, DomainError, NewAuditEvent, NewBackupSchedule, Permission, PitrStream,
    StorageTarget, UpdateBackupSchedule,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct CreateBackupScheduleRequest {
    pub name: String,
    pub cron_expression: String,
    pub backup_method: String,
    pub storage_target: StorageTarget,
    pub retention_days: u32,
    pub compression: CompressionFormat,
    #[serde(default = "default_true")]
    pub encrypt: bool,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct PitrStreamResponse {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub storage_path: String,
    pub start_time: DateTime<Utc>,
    pub latest_time: DateTime<Utc>,
    pub start_lsn_or_gtid: String,
    pub latest_lsn_or_gtid: String,
    pub status: String,
    pub rpo_lag_seconds: i64,
    pub updated_at: DateTime<Utc>,
}

impl From<PitrStream> for PitrStreamResponse {
    fn from(s: PitrStream) -> Self {
        let rpo_lag_seconds = s.calculate_rpo_lag_seconds();
        Self {
            id: s.id,
            cluster_id: s.cluster_id,
            storage_path: s.storage_path,
            start_time: s.start_time,
            latest_time: s.latest_time,
            start_lsn_or_gtid: s.start_lsn_or_gtid,
            latest_lsn_or_gtid: s.latest_lsn_or_gtid,
            status: s.status,
            rpo_lag_seconds,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpsertPitrStreamRequest {
    pub storage_path: String,
    pub start_time: Option<DateTime<Utc>>,
    pub latest_time: Option<DateTime<Utc>>,
    pub start_lsn_or_gtid: Option<String>,
    pub latest_lsn_or_gtid: Option<String>,
    pub status: Option<String>,
}

pub async fn list_cluster_backup_schedules(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(cluster_id): Path<Uuid>,
) -> Result<Json<Vec<BackupSchedule>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    let schedules = state
        .backup_schedule_repo
        .list_schedules_by_cluster(cluster_id)
        .await?;

    Ok(Json(schedules))
}

pub async fn create_cluster_backup_schedule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(cluster_id): Path<Uuid>,
    Json(payload): Json<CreateBackupScheduleRequest>,
) -> Result<(StatusCode, Json<BackupSchedule>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    // Verify cluster exists
    state
        .cluster_repo
        .get_cluster_by_id(cluster_id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("cluster not found: {cluster_id}")))?;

    let new_schedule = NewBackupSchedule {
        cluster_id,
        name: payload.name,
        cron_expression: payload.cron_expression,
        backup_method: payload.backup_method,
        storage_target: payload.storage_target,
        retention_days: payload.retention_days,
        compression: payload.compression,
        encrypt: payload.encrypt,
        is_active: payload.is_active,
    };

    let schedule = state
        .backup_schedule_repo
        .create_schedule(&new_schedule)
        .await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "backup_schedule.create".to_string(),
            resource_type: "backup_schedule".to_string(),
            resource_id: Some(schedule.id),
            details: Some(serde_json::json!({
                "cluster_id": cluster_id,
                "name": schedule.name,
                "cron": schedule.cron_expression,
                "target": schedule.storage_target.as_str(),
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::CREATED, Json(schedule)))
}

pub async fn get_backup_schedule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<BackupSchedule>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    let schedule = state
        .backup_schedule_repo
        .get_schedule_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("backup schedule '{id}' not found")))?;

    Ok(Json(schedule))
}

pub async fn update_backup_schedule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBackupSchedule>,
) -> Result<Json<BackupSchedule>, ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    let schedule = state
        .backup_schedule_repo
        .update_schedule(id, &payload)
        .await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "backup_schedule.update".to_string(),
            resource_type: "backup_schedule".to_string(),
            resource_id: Some(id),
            details: Some(serde_json::json!({
                "name": schedule.name,
                "is_active": schedule.is_active,
            })),
            source_ip: None,
        })
        .await;

    Ok(Json(schedule))
}

pub async fn delete_backup_schedule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    state.backup_schedule_repo.delete_schedule(id).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "backup_schedule.delete".to_string(),
            resource_type: "backup_schedule".to_string(),
            resource_id: Some(id),
            details: None,
            source_ip: None,
        })
        .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_cluster_pitr_stream(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(cluster_id): Path<Uuid>,
) -> Result<Json<Option<PitrStreamResponse>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    let stream = state
        .backup_schedule_repo
        .get_pitr_stream_by_cluster(cluster_id)
        .await?;

    Ok(Json(stream.map(PitrStreamResponse::from)))
}

pub async fn upsert_cluster_pitr_stream(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(cluster_id): Path<Uuid>,
    Json(payload): Json<UpsertPitrStreamRequest>,
) -> Result<Json<PitrStreamResponse>, ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    // Verify cluster exists
    state
        .cluster_repo
        .get_cluster_by_id(cluster_id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("cluster not found: {cluster_id}")))?;

    let existing = state
        .backup_schedule_repo
        .get_pitr_stream_by_cluster(cluster_id)
        .await?;

    let now = Utc::now();
    let stream = PitrStream {
        id: existing.as_ref().map(|e| e.id).unwrap_or_else(Uuid::now_v7),
        cluster_id,
        storage_path: payload.storage_path,
        start_time: payload
            .start_time
            .unwrap_or(existing.as_ref().map(|e| e.start_time).unwrap_or(now)),
        latest_time: payload.latest_time.unwrap_or(now),
        start_lsn_or_gtid: payload.start_lsn_or_gtid.unwrap_or_default(),
        latest_lsn_or_gtid: payload.latest_lsn_or_gtid.unwrap_or_default(),
        status: payload.status.unwrap_or_else(|| "active".to_string()),
        updated_at: now,
    };

    let saved = state
        .backup_schedule_repo
        .upsert_pitr_stream(&stream)
        .await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "pitr_stream.upsert".to_string(),
            resource_type: "pitr_stream".to_string(),
            resource_id: Some(saved.id),
            details: Some(serde_json::json!({
                "cluster_id": cluster_id,
                "storage_path": saved.storage_path,
                "status": saved.status,
            })),
            source_ip: None,
        })
        .await;

    Ok(Json(PitrStreamResponse::from(saved)))
}
