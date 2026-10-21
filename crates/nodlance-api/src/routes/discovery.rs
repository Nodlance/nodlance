//! Workload discovery query and trigger route handlers.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use nodlance_domain::{
    AuditRepository, DiscoveredWorkload, DiscoveryRepository, DomainError, Job, JobRepository,
    JobType, NewAuditEvent, NewJob, Permission, ProviderType,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct TriggerScanRequest {
    pub provider_type: Option<ProviderType>,
    pub namespace: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListDiscoveryQuery {
    pub provider_type: Option<ProviderType>,
}

pub async fn trigger_scan(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<Option<TriggerScanRequest>>,
) -> Result<(StatusCode, Json<Job>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    let req = payload.unwrap_or(TriggerScanRequest {
        provider_type: Some(ProviderType::Kubernetes),
        namespace: Some("default".to_string()),
    });

    let p_type = req.provider_type.unwrap_or(ProviderType::Kubernetes);
    let idempotency_key = format!("disc-{}-{}", p_type.as_str(), Uuid::now_v7());

    let job = state
        .job_repo
        .enqueue_job(&NewJob {
            job_type: JobType::DiscoverWorkloads,
            idempotency_key,
            payload: serde_json::json!({
                "provider_type": p_type.as_str(),
                "namespace": req.namespace,
                "triggered_by": auth.0.user_id.to_string(),
            }),
            max_attempts: Some(2),
        })
        .await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "discovery.scan.trigger".to_string(),
            resource_type: "workload_discovery".to_string(),
            resource_id: None,
            details: Some(serde_json::json!({
                "job_id": job.id,
                "provider_type": p_type.as_str(),
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::ACCEPTED, Json(job)))
}

pub async fn list_discovered_workloads(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListDiscoveryQuery>,
) -> Result<Json<Vec<DiscoveredWorkload>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let workloads = state
        .discovery_repo
        .list_discovered_workloads(query.provider_type)
        .await?;
    Ok(Json(workloads))
}

pub async fn get_discovered_workload(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<DiscoveredWorkload>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;
    let workload = state
        .discovery_repo
        .get_discovered_workload_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("discovered workload not found: {id}")))?;

    Ok(Json(workload))
}
