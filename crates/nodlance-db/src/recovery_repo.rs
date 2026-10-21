//! PostgreSQL implementation of RecoveryRepository.

use chrono::{DateTime, Utc};
use nodlance_domain::{
    ClusterEpoch, DomainError, FencingExecution, FencingStatus, FencingType, NewPromotionRequest,
    PromotionRequest, PromotionStatus, RecoveryRepository, UpdatePromotionParams,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgRecoveryRepository {
    pool: PgPool,
}

impl PgRecoveryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct ClusterEpochRow {
    id: Uuid,
    cluster_id: Uuid,
    epoch: i64,
    promoted_host_id: Option<Uuid>,
    reason: String,
    created_at: DateTime<Utc>,
}

impl From<ClusterEpochRow> for ClusterEpoch {
    fn from(row: ClusterEpochRow) -> Self {
        Self {
            id: row.id,
            cluster_id: row.cluster_id,
            epoch: row.epoch as u64,
            promoted_host_id: row.promoted_host_id,
            reason: row.reason,
            created_at: row.created_at,
        }
    }
}

#[derive(FromRow)]
struct FencingExecutionRow {
    id: Uuid,
    cluster_id: Uuid,
    target_host_id: Uuid,
    fencing_type: String,
    status: String,
    verification_proof: Option<String>,
    error_message: Option<String>,
    executed_at: DateTime<Utc>,
    verified_at: Option<DateTime<Utc>>,
}

impl TryFrom<FencingExecutionRow> for FencingExecution {
    type Error = DomainError;

    fn try_from(row: FencingExecutionRow) -> Result<Self, Self::Error> {
        let fencing_type = row
            .fencing_type
            .parse::<FencingType>()
            .map_err(|e| DomainError::Internal(format!("invalid fencing_type in db: {e}")))?;

        let status = row
            .status
            .parse::<FencingStatus>()
            .map_err(|e| DomainError::Internal(format!("invalid fencing_status in db: {e}")))?;

        Ok(Self {
            id: row.id,
            cluster_id: row.cluster_id,
            target_host_id: row.target_host_id,
            fencing_type,
            status,
            verification_proof: row.verification_proof,
            error_message: row.error_message,
            executed_at: row.executed_at,
            verified_at: row.verified_at,
        })
    }
}

#[derive(FromRow)]
struct PromotionRequestRow {
    id: Uuid,
    cluster_id: Uuid,
    candidate_host_id: Uuid,
    target_epoch: i64,
    status: String,
    approval_token: Option<String>,
    approved_by: Option<Uuid>,
    approved_at: Option<DateTime<Utc>>,
    fencing_execution_id: Option<Uuid>,
    safety_gates_passed: bool,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<PromotionRequestRow> for PromotionRequest {
    type Error = DomainError;

    fn try_from(row: PromotionRequestRow) -> Result<Self, Self::Error> {
        let status = row
            .status
            .parse::<PromotionStatus>()
            .map_err(|e| DomainError::Internal(format!("invalid promotion status in db: {e}")))?;

        Ok(Self {
            id: row.id,
            cluster_id: row.cluster_id,
            candidate_host_id: row.candidate_host_id,
            target_epoch: row.target_epoch as u64,
            status,
            approval_token: row.approval_token,
            approved_by: row.approved_by,
            approved_at: row.approved_at,
            fencing_execution_id: row.fencing_execution_id,
            safety_gates_passed: row.safety_gates_passed,
            error_message: row.error_message,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl RecoveryRepository for PgRecoveryRepository {
    async fn get_current_epoch(
        &self,
        cluster_id: Uuid,
    ) -> Result<Option<ClusterEpoch>, DomainError> {
        let row = sqlx::query_as::<_, ClusterEpochRow>(
            r#"
            SELECT id, cluster_id, epoch, promoted_host_id, reason, created_at
            FROM cluster_epochs
            WHERE cluster_id = $1
            ORDER BY epoch DESC
            LIMIT 1
            "#,
        )
        .bind(cluster_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "failed to get current cluster epoch"))?;

        Ok(row.map(ClusterEpoch::from))
    }

    async fn increment_epoch(
        &self,
        cluster_id: Uuid,
        promoted_host_id: Option<Uuid>,
        reason: &str,
    ) -> Result<ClusterEpoch, DomainError> {
        let current = self.get_current_epoch(cluster_id).await?;
        let next_epoch = current.map_or(1, |c| c.epoch + 1);

        let id = Uuid::now_v7();
        let now = Utc::now();

        let row = sqlx::query_as::<_, ClusterEpochRow>(
            r#"
            INSERT INTO cluster_epochs (id, cluster_id, epoch, promoted_host_id, reason, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, cluster_id, epoch, promoted_host_id, reason, created_at
            "#,
        )
        .bind(id)
        .bind(cluster_id)
        .bind(next_epoch as i64)
        .bind(promoted_host_id)
        .bind(reason)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "failed to increment cluster epoch"))?;

        Ok(ClusterEpoch::from(row))
    }

    async fn create_promotion_request(
        &self,
        req: &NewPromotionRequest,
    ) -> Result<PromotionRequest, DomainError> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let status = PromotionStatus::PendingApproval.as_str();

        let row = sqlx::query_as::<_, PromotionRequestRow>(
            r#"
            INSERT INTO promotion_requests (
                id, cluster_id, candidate_host_id, target_epoch, status,
                approval_token, approved_by, approved_at, fencing_execution_id,
                safety_gates_passed, error_message, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, NULL, NULL, NULL, NULL, FALSE, NULL, $6, $7)
            RETURNING id, cluster_id, candidate_host_id, target_epoch, status,
                      approval_token, approved_by, approved_at, fencing_execution_id,
                      safety_gates_passed, error_message, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(req.cluster_id)
        .bind(req.candidate_host_id)
        .bind(req.target_epoch as i64)
        .bind(status)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "failed to create promotion request"))?;

        PromotionRequest::try_from(row)
    }

    async fn get_promotion_request(
        &self,
        id: Uuid,
    ) -> Result<Option<PromotionRequest>, DomainError> {
        let row = sqlx::query_as::<_, PromotionRequestRow>(
            r#"
            SELECT id, cluster_id, candidate_host_id, target_epoch, status,
                   approval_token, approved_by, approved_at, fencing_execution_id,
                   safety_gates_passed, error_message, created_at, updated_at
            FROM promotion_requests
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "failed to get promotion request"))?;

        row.map(PromotionRequest::try_from).transpose()
    }

    async fn list_promotion_requests(
        &self,
        cluster_id: Option<Uuid>,
    ) -> Result<Vec<PromotionRequest>, DomainError> {
        let rows = if let Some(cid) = cluster_id {
            sqlx::query_as::<_, PromotionRequestRow>(
                r#"
                SELECT id, cluster_id, candidate_host_id, target_epoch, status,
                       approval_token, approved_by, approved_at, fencing_execution_id,
                       safety_gates_passed, error_message, created_at, updated_at
                FROM promotion_requests
                WHERE cluster_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(cid)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, PromotionRequestRow>(
                r#"
                SELECT id, cluster_id, candidate_host_id, target_epoch, status,
                       approval_token, approved_by, approved_at, fencing_execution_id,
                       safety_gates_passed, error_message, created_at, updated_at
                FROM promotion_requests
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| map_sqlx_error(e, "failed to list promotion requests"))?;

        rows.into_iter().map(PromotionRequest::try_from).collect()
    }

    async fn update_promotion_status(
        &self,
        id: Uuid,
        params: UpdatePromotionParams<'_>,
    ) -> Result<PromotionRequest, DomainError> {
        let now = Utc::now();
        let approved_at = if params.status == PromotionStatus::Approved {
            Some(now)
        } else {
            None
        };

        let row = sqlx::query_as::<_, PromotionRequestRow>(
            r#"
            UPDATE promotion_requests
            SET status = $2,
                approval_token = COALESCE($3, approval_token),
                approved_by = COALESCE($4, approved_by),
                approved_at = COALESCE($5, approved_at),
                fencing_execution_id = COALESCE($6, fencing_execution_id),
                safety_gates_passed = COALESCE($7, safety_gates_passed),
                error_message = COALESCE($8, error_message),
                updated_at = $9
            WHERE id = $1
            RETURNING id, cluster_id, candidate_host_id, target_epoch, status,
                      approval_token, approved_by, approved_at, fencing_execution_id,
                      safety_gates_passed, error_message, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(params.status.as_str())
        .bind(params.approval_token)
        .bind(params.approved_by)
        .bind(approved_at)
        .bind(params.fencing_id)
        .bind(params.safety_gates_passed)
        .bind(params.error_message)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "failed to update promotion status"))?;

        PromotionRequest::try_from(row)
    }

    async fn record_fencing_execution(
        &self,
        execution: &FencingExecution,
    ) -> Result<FencingExecution, DomainError> {
        let row = sqlx::query_as::<_, FencingExecutionRow>(
            r#"
            INSERT INTO fencing_executions (
                id, cluster_id, target_host_id, fencing_type, status,
                verification_proof, error_message, executed_at, verified_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, cluster_id, target_host_id, fencing_type, status,
                      verification_proof, error_message, executed_at, verified_at
            "#,
        )
        .bind(execution.id)
        .bind(execution.cluster_id)
        .bind(execution.target_host_id)
        .bind(execution.fencing_type.as_str())
        .bind(execution.status.as_str())
        .bind(&execution.verification_proof)
        .bind(&execution.error_message)
        .bind(execution.executed_at)
        .bind(execution.verified_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "failed to record fencing execution"))?;

        FencingExecution::try_from(row)
    }

    async fn get_fencing_execution(
        &self,
        id: Uuid,
    ) -> Result<Option<FencingExecution>, DomainError> {
        let row = sqlx::query_as::<_, FencingExecutionRow>(
            r#"
            SELECT id, cluster_id, target_host_id, fencing_type, status,
                   verification_proof, error_message, executed_at, verified_at
            FROM fencing_executions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "failed to get fencing execution"))?;

        row.map(FencingExecution::try_from).transpose()
    }

    async fn update_fencing_status(
        &self,
        id: Uuid,
        status: FencingStatus,
        verification_proof: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<FencingExecution, DomainError> {
        let now = Utc::now();
        let verified_at = if status == FencingStatus::Verified {
            Some(now)
        } else {
            None
        };

        let row = sqlx::query_as::<_, FencingExecutionRow>(
            r#"
            UPDATE fencing_executions
            SET status = $2,
                verification_proof = COALESCE($3, verification_proof),
                error_message = COALESCE($4, error_message),
                verified_at = COALESCE($5, verified_at)
            WHERE id = $1
            RETURNING id, cluster_id, target_host_id, fencing_type, status,
                      verification_proof, error_message, executed_at, verified_at
            "#,
        )
        .bind(id)
        .bind(status.as_str())
        .bind(verification_proof)
        .bind(error_message)
        .bind(verified_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "failed to update fencing status"))?;

        FencingExecution::try_from(row)
    }
}
