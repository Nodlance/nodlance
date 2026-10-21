//! Backup workflow and verification route handlers.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use nodlance_domain::{
    AuditRepository, BackupArtifact, BackupRepository, ClusterRepository, DomainError, Job,
    JobRepository, JobType, NewAuditEvent, NewJob, Permission, RestoreValidationDrill,
};
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, state::AppState};

pub async fn trigger_cluster_backup(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(cluster_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Job>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    // Verify cluster exists
    state
        .cluster_repo
        .get_cluster_by_id(cluster_id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("cluster not found: {cluster_id}")))?;

    let idempotency_key = format!("backup-{cluster_id}-{}", Uuid::now_v7());
    let job = state
        .job_repo
        .enqueue_job(&NewJob {
            job_type: JobType::BackupCluster,
            idempotency_key,
            payload: serde_json::json!({
                "cluster_id": cluster_id.to_string(),
                "triggered_by": auth.0.user_id.to_string(),
            }),
            max_attempts: Some(3),
        })
        .await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "cluster.backup.trigger".to_string(),
            resource_type: "cluster".to_string(),
            resource_id: Some(cluster_id),
            details: Some(serde_json::json!({
                "job_id": job.id,
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::ACCEPTED, Json(job)))
}

pub async fn list_cluster_backups(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(cluster_id): Path<Uuid>,
) -> Result<Json<Vec<BackupArtifact>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let backups = state
        .backup_repo
        .list_backups_by_cluster(cluster_id)
        .await?;
    Ok(Json(backups))
}

pub async fn get_backup_details(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<BackupArtifact>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let backup = state
        .backup_repo
        .get_backup_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("backup artifact not found: {id}")))?;

    Ok(Json(backup))
}

pub async fn verify_backup(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Job>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    // Verify backup exists
    state
        .backup_repo
        .get_backup_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("backup artifact not found: {id}")))?;

    let idempotency_key = format!("drill-{id}-{}", Uuid::now_v7());
    let job = state
        .job_repo
        .enqueue_job(&NewJob {
            job_type: JobType::RestoreDrill,
            idempotency_key,
            payload: serde_json::json!({
                "backup_id": id.to_string(),
                "triggered_by": auth.0.user_id.to_string(),
            }),
            max_attempts: Some(2),
        })
        .await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "backup.verify.trigger".to_string(),
            resource_type: "backup".to_string(),
            resource_id: Some(id),
            details: Some(serde_json::json!({
                "job_id": job.id,
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::ACCEPTED, Json(job)))
}

pub async fn get_latest_drill(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<RestoreValidationDrill>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let drill = state.backup_repo.get_latest_drill(id).await?;
    Ok(Json(drill))
}
