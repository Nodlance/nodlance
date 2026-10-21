//! PostgreSQL implementation of JobRepository with atomic leasing.

use chrono::{DateTime, Utc};
use nodlance_domain::{DomainError, Job, JobAttempt, JobRepository, JobStatus, JobType, NewJob};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgJobRepository {
    pool: PgPool,
}

impl PgJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct JobRow {
    id: Uuid,
    job_type: String,
    status: String,
    idempotency_key: String,
    payload: serde_json::Value,
    result: Option<serde_json::Value>,
    error_message: Option<String>,
    lease_owner: Option<String>,
    lease_expires_at: Option<DateTime<Utc>>,
    attempt_count: i32,
    max_attempts: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<JobRow> for Job {
    type Error = DomainError;

    fn try_from(row: JobRow) -> Result<Self, Self::Error> {
        let job_type = row
            .job_type
            .parse::<JobType>()
            .map_err(|e| DomainError::Internal(format!("invalid job type in db: {e}")))?;

        let status = match row.status.as_str() {
            "queued" => JobStatus::Queued,
            "running" => JobStatus::Running,
            "succeeded" => JobStatus::Succeeded,
            "failed" => JobStatus::Failed,
            "cancelled" => JobStatus::Cancelled,
            "unknown_outcome" => JobStatus::UnknownOutcome,
            "reconciliation_required" => JobStatus::ReconciliationRequired,
            other => {
                return Err(DomainError::Internal(format!(
                    "invalid job status in db: {other}"
                )));
            }
        };

        Ok(Self {
            id: row.id,
            job_type,
            status,
            idempotency_key: row.idempotency_key,
            payload: row.payload,
            result: row.result,
            error_message: row.error_message,
            lease_owner: row.lease_owner,
            lease_expires_at: row.lease_expires_at,
            attempt_count: row.attempt_count,
            max_attempts: row.max_attempts,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(FromRow)]
struct AttemptRow {
    id: Uuid,
    job_id: Uuid,
    attempt_number: i32,
    status: String,
    error_message: Option<String>,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
}

impl TryFrom<AttemptRow> for JobAttempt {
    type Error = DomainError;

    fn try_from(row: AttemptRow) -> Result<Self, Self::Error> {
        let status = match row.status.as_str() {
            "running" => JobStatus::Running,
            "succeeded" => JobStatus::Succeeded,
            "failed" => JobStatus::Failed,
            _ => JobStatus::UnknownOutcome,
        };

        Ok(Self {
            id: row.id,
            job_id: row.job_id,
            attempt_number: row.attempt_number,
            status,
            error_message: row.error_message,
            started_at: row.started_at,
            finished_at: row.finished_at,
        })
    }
}

impl JobRepository for PgJobRepository {
    async fn enqueue_job(&self, new_job: &NewJob) -> Result<Job, DomainError> {
        let id = Uuid::now_v7();
        let max_attempts = new_job.max_attempts.unwrap_or(3).max(1);

        let row = sqlx::query_as::<_, JobRow>(
            r#"
            INSERT INTO jobs (id, job_type, status, idempotency_key, payload, attempt_count, max_attempts, created_at, updated_at)
            VALUES ($1, $2, 'queued', $3, $4, 0, $5, now(), now())
            ON CONFLICT (idempotency_key) DO UPDATE
            SET updated_at = now()
            RETURNING id, job_type, status, idempotency_key, payload, result, error_message, lease_owner, lease_expires_at, attempt_count, max_attempts, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(new_job.job_type.as_str())
        .bind(&new_job.idempotency_key)
        .bind(&new_job.payload)
        .bind(max_attempts)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "enqueue job"))?;

        row.try_into()
    }

    async fn acquire_lease(
        &self,
        lease_owner: &str,
        lease_duration_secs: u64,
    ) -> Result<Option<Job>, DomainError> {
        let row = sqlx::query_as::<_, JobRow>(
            r#"
            UPDATE jobs
            SET status = 'running',
                lease_owner = $1,
                lease_expires_at = now() + ($2 || ' seconds')::INTERVAL,
                attempt_count = attempt_count + 1,
                updated_at = now()
            WHERE id = (
                SELECT id FROM jobs
                WHERE (status = 'queued' OR (status = 'running' AND lease_expires_at < now()))
                  AND attempt_count < max_attempts
                ORDER BY created_at ASC
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            RETURNING id, job_type, status, idempotency_key, payload, result, error_message, lease_owner, lease_expires_at, attempt_count, max_attempts, created_at, updated_at
            "#,
        )
        .bind(lease_owner)
        .bind(lease_duration_secs.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "acquire job lease"))?;

        row.map(Job::try_from).transpose()
    }

    async fn record_attempt_start(
        &self,
        job_id: Uuid,
        attempt_number: i32,
    ) -> Result<JobAttempt, DomainError> {
        let id = Uuid::now_v7();

        let row = sqlx::query_as::<_, AttemptRow>(
            r#"
            INSERT INTO job_attempts (id, job_id, attempt_number, status, started_at)
            VALUES ($1, $2, $3, 'running', now())
            RETURNING id, job_id, attempt_number, status, error_message, started_at, finished_at
            "#,
        )
        .bind(id)
        .bind(job_id)
        .bind(attempt_number)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "record job attempt start"))?;

        row.try_into()
    }

    async fn complete_job(
        &self,
        job_id: Uuid,
        attempt_id: Uuid,
        result: serde_json::Value,
    ) -> Result<Job, DomainError> {
        // Complete attempt
        let _ = sqlx::query(
            r#"
            UPDATE job_attempts
            SET status = 'succeeded', finished_at = now()
            WHERE id = $1
            "#,
        )
        .bind(attempt_id)
        .execute(&self.pool)
        .await;

        // Complete job
        let row = sqlx::query_as::<_, JobRow>(
            r#"
            UPDATE jobs
            SET status = 'succeeded',
                result = $2,
                error_message = NULL,
                lease_owner = NULL,
                lease_expires_at = NULL,
                updated_at = now()
            WHERE id = $1
            RETURNING id, job_type, status, idempotency_key, payload, result, error_message, lease_owner, lease_expires_at, attempt_count, max_attempts, created_at, updated_at
            "#,
        )
        .bind(job_id)
        .bind(result)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "complete job"))?;

        row.try_into()
    }

    async fn fail_job(
        &self,
        job_id: Uuid,
        attempt_id: Uuid,
        error_message: &str,
        can_retry: bool,
    ) -> Result<Job, DomainError> {
        // Fail attempt
        let _ = sqlx::query(
            r#"
            UPDATE job_attempts
            SET status = 'failed', error_message = $2, finished_at = now()
            WHERE id = $1
            "#,
        )
        .bind(attempt_id)
        .bind(error_message)
        .execute(&self.pool)
        .await;

        let new_status = if can_retry { "queued" } else { "failed" };

        let row = sqlx::query_as::<_, JobRow>(
            r#"
            UPDATE jobs
            SET status = CASE WHEN attempt_count >= max_attempts THEN 'failed' ELSE $2 END,
                error_message = $3,
                lease_owner = NULL,
                lease_expires_at = NULL,
                updated_at = now()
            WHERE id = $1
            RETURNING id, job_type, status, idempotency_key, payload, result, error_message, lease_owner, lease_expires_at, attempt_count, max_attempts, created_at, updated_at
            "#,
        )
        .bind(job_id)
        .bind(new_status)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "fail job"))?;

        row.try_into()
    }

    async fn get_job_by_id(&self, job_id: Uuid) -> Result<Option<Job>, DomainError> {
        let row = sqlx::query_as::<_, JobRow>(
            r#"
            SELECT id, job_type, status, idempotency_key, payload, result, error_message, lease_owner, lease_expires_at, attempt_count, max_attempts, created_at, updated_at
            FROM jobs
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get job by id"))?;

        row.map(Job::try_from).transpose()
    }

    async fn list_jobs(&self, limit: i64) -> Result<Vec<Job>, DomainError> {
        let limit = limit.clamp(1, 100);

        let rows = sqlx::query_as::<_, JobRow>(
            r#"
            SELECT id, job_type, status, idempotency_key, payload, result, error_message, lease_owner, lease_expires_at, attempt_count, max_attempts, created_at, updated_at
            FROM jobs
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list jobs"))?;

        rows.into_iter().map(Job::try_from).collect()
    }
}
