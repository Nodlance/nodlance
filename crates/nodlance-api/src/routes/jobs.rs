//! Durable background jobs query route handlers.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use nodlance_domain::{DomainError, Job, JobRepository, Permission};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    pub limit: Option<i64>,
}

pub async fn list_jobs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListJobsQuery>,
) -> Result<Json<Vec<Job>>, ApiError> {
    if !auth.0.has_permission(Permission::AuditRead) && !auth.0.has_permission(Permission::HostRead)
    {
        return Err(
            nodlance_auth::AuthError::Forbidden("audit:read or host:read".to_string()).into(),
        );
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let jobs = state.job_repo.list_jobs(limit).await?;
    Ok(Json(jobs))
}

pub async fn get_job(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Job>, ApiError> {
    if !auth.0.has_permission(Permission::AuditRead) && !auth.0.has_permission(Permission::HostRead)
    {
        return Err(
            nodlance_auth::AuthError::Forbidden("audit:read or host:read".to_string()).into(),
        );
    }

    let job = state
        .job_repo
        .get_job_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("job not found: {id}")))?;

    Ok(Json(job))
}
