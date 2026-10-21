//! Guarded provisioning plan management and execution route handlers.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use nodlance_domain::{
    AuditRepository, ClusterRepository, DomainError, Job, JobRepository, JobType, NewAuditEvent,
    NewJob, NewProvisioningPlan, Permission, ProvisioningPlan, ProvisioningRepository,
    ProvisioningStep,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListPlansQuery {
    pub cluster_id: Option<Uuid>,
}

pub async fn create_plan(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<NewProvisioningPlan>,
) -> Result<(StatusCode, Json<ProvisioningPlan>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    // Verify cluster exists
    state
        .cluster_repo
        .get_cluster_by_id(payload.cluster_id)
        .await?
        .ok_or_else(|| {
            DomainError::NotFound(format!("cluster not found: {}", payload.cluster_id))
        })?;

    let plan = state.provisioning_repo.create_plan(&payload).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "provisioning.plan.create".to_string(),
            resource_type: "provisioning_plan".to_string(),
            resource_id: Some(plan.id),
            details: Some(serde_json::json!({
                "cluster_id": plan.cluster_id,
                "provider_type": plan.provider_type.as_str(),
                "engine": plan.engine.as_str(),
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::CREATED, Json(plan)))
}

pub async fn list_plans(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListPlansQuery>,
) -> Result<Json<Vec<ProvisioningPlan>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let plans = state.provisioning_repo.list_plans(query.cluster_id).await?;
    Ok(Json(plans))
}

pub async fn get_plan(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ProvisioningPlan>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let plan = state
        .provisioning_repo
        .get_plan_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("provisioning plan not found: {id}")))?;

    Ok(Json(plan))
}

pub async fn list_plan_steps(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ProvisioningStep>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    // Verify plan exists
    state
        .provisioning_repo
        .get_plan_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("provisioning plan not found: {id}")))?;

    let steps = state.provisioning_repo.list_steps_for_plan(id).await?;
    Ok(Json(steps))
}

pub async fn execute_plan(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Job>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    // Verify plan exists
    let plan = state
        .provisioning_repo
        .get_plan_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("provisioning plan not found: {id}")))?;

    let idempotency_key = format!("prov-{id}-{}", Uuid::now_v7());
    let job = state
        .job_repo
        .enqueue_job(&NewJob {
            job_type: JobType::ProvisionCluster,
            idempotency_key,
            payload: serde_json::json!({
                "plan_id": id.to_string(),
                "cluster_id": plan.cluster_id.to_string(),
                "triggered_by": auth.0.user_id.to_string(),
            }),
            max_attempts: Some(1), // Guarded mutations do not blindly retry side-effects
        })
        .await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "provisioning.plan.execute".to_string(),
            resource_type: "provisioning_plan".to_string(),
            resource_id: Some(id),
            details: Some(serde_json::json!({
                "job_id": job.id,
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::ACCEPTED, Json(job)))
}

pub async fn rollback_plan(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Job>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    // Verify plan exists
    state
        .provisioning_repo
        .get_plan_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("provisioning plan not found: {id}")))?;

    let idempotency_key = format!("rollback-{id}-{}", Uuid::now_v7());
    let job = state
        .job_repo
        .enqueue_job(&NewJob {
            job_type: JobType::RollbackProvisioning,
            idempotency_key,
            payload: serde_json::json!({
                "plan_id": id.to_string(),
                "triggered_by": auth.0.user_id.to_string(),
            }),
            max_attempts: Some(1),
        })
        .await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "provisioning.plan.rollback".to_string(),
            resource_type: "provisioning_plan".to_string(),
            resource_id: Some(id),
            details: Some(serde_json::json!({
                "job_id": job.id,
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::ACCEPTED, Json(job)))
}
