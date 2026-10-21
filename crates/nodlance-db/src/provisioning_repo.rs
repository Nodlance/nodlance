//! PostgreSQL implementations of ProvisioningRepository and DiscoveryRepository.

use chrono::{DateTime, Utc};
use nodlance_domain::{
    DatabaseEngine, DiscoveredWorkload, DiscoveryRepository, DomainError, NewDiscoveredWorkload,
    NewProvisioningPlan, ProviderType, ProvisioningConfig, ProvisioningPlan,
    ProvisioningRepository, ProvisioningStatus, ProvisioningStep, ProvisioningStepType, StepStatus,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgProvisioningRepository {
    pool: PgPool,
}

impl PgProvisioningRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct PlanRow {
    id: Uuid,
    cluster_id: Uuid,
    host_id: Option<Uuid>,
    provider_type: String,
    engine: String,
    engine_version: String,
    config: serde_json::Value,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<PlanRow> for ProvisioningPlan {
    type Error = DomainError;

    fn try_from(row: PlanRow) -> Result<Self, Self::Error> {
        let provider_type = row
            .provider_type
            .parse::<ProviderType>()
            .map_err(|e| DomainError::Internal(format!("invalid provider_type in db: {e}")))?;

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

        let status = row
            .status
            .parse::<ProvisioningStatus>()
            .map_err(|e| DomainError::Internal(format!("invalid status in db: {e}")))?;

        let config: ProvisioningConfig = serde_json::from_value(row.config)
            .map_err(|e| DomainError::Internal(format!("invalid config json in db: {e}")))?;

        Ok(Self {
            id: row.id,
            cluster_id: row.cluster_id,
            host_id: row.host_id,
            provider_type,
            engine,
            engine_version: row.engine_version,
            config,
            status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(FromRow)]
struct StepRow {
    id: Uuid,
    plan_id: Uuid,
    step_index: i32,
    step_type: String,
    status: String,
    error_details: Option<String>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

impl TryFrom<StepRow> for ProvisioningStep {
    type Error = DomainError;

    fn try_from(row: StepRow) -> Result<Self, Self::Error> {
        let step_type = row
            .step_type
            .parse::<ProvisioningStepType>()
            .map_err(|e| DomainError::Internal(format!("invalid step_type in db: {e}")))?;

        let status = row
            .status
            .parse::<StepStatus>()
            .map_err(|e| DomainError::Internal(format!("invalid step status in db: {e}")))?;

        Ok(Self {
            id: row.id,
            plan_id: row.plan_id,
            step_index: row.step_index,
            step_type,
            status,
            error_details: row.error_details,
            started_at: row.started_at,
            completed_at: row.completed_at,
        })
    }
}

impl ProvisioningRepository for PgProvisioningRepository {
    async fn create_plan(
        &self,
        plan: &NewProvisioningPlan,
    ) -> Result<ProvisioningPlan, DomainError> {
        let id = Uuid::now_v7();
        let config_json = serde_json::to_value(&plan.config)
            .map_err(|e| DomainError::Internal(format!("failed to serialize config: {e}")))?;

        let row = sqlx::query_as::<_, PlanRow>(
            r#"
            INSERT INTO provisioning_plans (
                id, cluster_id, host_id, provider_type, engine, engine_version, config, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING id, cluster_id, host_id, provider_type, engine, engine_version, config, status, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(plan.cluster_id)
        .bind(plan.host_id)
        .bind(plan.provider_type.as_str())
        .bind(plan.engine.as_str())
        .bind(&plan.engine_version)
        .bind(config_json)
        .bind(ProvisioningStatus::Pending.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "create provisioning plan"))?;

        // Seed default sequential steps for this plan
        let default_steps = [
            (0, ProvisioningStepType::PreflightCheck),
            (1, ProvisioningStepType::RenderConfiguration),
            (2, ProvisioningStepType::PrepareStorage),
            (3, ProvisioningStepType::InitializeDatabase),
            (4, ProvisioningStepType::StartService),
            (5, ProvisioningStepType::VerifyReadiness),
        ];

        for (idx, step_type) in default_steps {
            let step_id = Uuid::now_v7();
            let _ = sqlx::query(
                r#"
                INSERT INTO provisioning_steps (
                    id, plan_id, step_index, step_type, status
                )
                VALUES ($1, $2, $3, $4, 'pending')
                "#,
            )
            .bind(step_id)
            .bind(id)
            .bind(idx)
            .bind(step_type.as_str())
            .execute(&self.pool)
            .await;
        }

        row.try_into()
    }

    async fn get_plan_by_id(&self, id: Uuid) -> Result<Option<ProvisioningPlan>, DomainError> {
        let row = sqlx::query_as::<_, PlanRow>(
            r#"
            SELECT id, cluster_id, host_id, provider_type, engine, engine_version, config, status, created_at, updated_at
            FROM provisioning_plans
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get provisioning plan by id"))?;

        row.map(TryInto::try_into).transpose()
    }

    async fn list_plans(
        &self,
        cluster_id: Option<Uuid>,
    ) -> Result<Vec<ProvisioningPlan>, DomainError> {
        let rows = if let Some(cid) = cluster_id {
            sqlx::query_as::<_, PlanRow>(
                r#"
                SELECT id, cluster_id, host_id, provider_type, engine, engine_version, config, status, created_at, updated_at
                FROM provisioning_plans
                WHERE cluster_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(cid)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_sqlx_error(e, "list provisioning plans by cluster"))?
        } else {
            sqlx::query_as::<_, PlanRow>(
                r#"
                SELECT id, cluster_id, host_id, provider_type, engine, engine_version, config, status, created_at, updated_at
                FROM provisioning_plans
                ORDER BY created_at DESC
                LIMIT 100
                "#,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_sqlx_error(e, "list provisioning plans"))?
        };

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn update_plan_status(
        &self,
        id: Uuid,
        status: ProvisioningStatus,
    ) -> Result<ProvisioningPlan, DomainError> {
        let row = sqlx::query_as::<_, PlanRow>(
            r#"
            UPDATE provisioning_plans
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, cluster_id, host_id, provider_type, engine, engine_version, config, status, created_at, updated_at
            "#,
        )
        .bind(status.as_str())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "update provisioning plan status"))?
        .ok_or_else(|| DomainError::NotFound(format!("provisioning plan not found: {id}")))?;

        row.try_into()
    }

    async fn record_step(&self, step: &ProvisioningStep) -> Result<ProvisioningStep, DomainError> {
        let row = sqlx::query_as::<_, StepRow>(
            r#"
            INSERT INTO provisioning_steps (
                id, plan_id, step_index, step_type, status, error_details, started_at, completed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (plan_id, step_index) DO UPDATE
            SET status = EXCLUDED.status,
                error_details = EXCLUDED.error_details,
                started_at = COALESCE(provisioning_steps.started_at, EXCLUDED.started_at),
                completed_at = EXCLUDED.completed_at
            RETURNING id, plan_id, step_index, step_type, status, error_details, started_at, completed_at
            "#,
        )
        .bind(step.id)
        .bind(step.plan_id)
        .bind(step.step_index)
        .bind(step.step_type.as_str())
        .bind(step.status.as_str())
        .bind(&step.error_details)
        .bind(step.started_at)
        .bind(step.completed_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "record provisioning step"))?;

        row.try_into()
    }

    async fn list_steps_for_plan(
        &self,
        plan_id: Uuid,
    ) -> Result<Vec<ProvisioningStep>, DomainError> {
        let rows = sqlx::query_as::<_, StepRow>(
            r#"
            SELECT id, plan_id, step_index, step_type, status, error_details, started_at, completed_at
            FROM provisioning_steps
            WHERE plan_id = $1
            ORDER BY step_index ASC
            "#,
        )
        .bind(plan_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list steps for plan"))?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn update_step_status(
        &self,
        step_id: Uuid,
        status: StepStatus,
        error_details: Option<&str>,
    ) -> Result<ProvisioningStep, DomainError> {
        let completed_at = match status {
            StepStatus::Succeeded | StepStatus::Failed | StepStatus::RolledBack => Some(Utc::now()),
            _ => None,
        };

        let row = sqlx::query_as::<_, StepRow>(
            r#"
            UPDATE provisioning_steps
            SET status = $1,
                error_details = $2,
                completed_at = COALESCE(completed_at, $3),
                started_at = COALESCE(started_at, NOW())
            WHERE id = $4
            RETURNING id, plan_id, step_index, step_type, status, error_details, started_at, completed_at
            "#,
        )
        .bind(status.as_str())
        .bind(error_details)
        .bind(completed_at)
        .bind(step_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "update step status"))?
        .ok_or_else(|| DomainError::NotFound(format!("step not found: {step_id}")))?;

        row.try_into()
    }
}

#[derive(Clone)]
pub struct PgDiscoveryRepository {
    pool: PgPool,
}

impl PgDiscoveryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct WorkloadRow {
    id: Uuid,
    provider_type: String,
    namespace: Option<String>,
    workload_name: String,
    engine: String,
    detected_version: Option<String>,
    endpoints: serde_json::Value,
    status: String,
    details: Option<serde_json::Value>,
    last_seen_at: DateTime<Utc>,
}

impl TryFrom<WorkloadRow> for DiscoveredWorkload {
    type Error = DomainError;

    fn try_from(row: WorkloadRow) -> Result<Self, Self::Error> {
        let provider_type = row
            .provider_type
            .parse::<ProviderType>()
            .map_err(|e| DomainError::Internal(format!("invalid provider_type in db: {e}")))?;

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

        let endpoints: Vec<String> = serde_json::from_value(row.endpoints).unwrap_or_default();

        Ok(Self {
            id: row.id,
            provider_type,
            namespace: row.namespace,
            workload_name: row.workload_name,
            engine,
            detected_version: row.detected_version,
            endpoints,
            status: row.status,
            details: row.details,
            last_seen_at: row.last_seen_at,
        })
    }
}

impl DiscoveryRepository for PgDiscoveryRepository {
    async fn record_discovered_workload(
        &self,
        workload: &NewDiscoveredWorkload,
    ) -> Result<DiscoveredWorkload, DomainError> {
        let id = Uuid::now_v7();
        let endpoints_json = serde_json::to_value(&workload.endpoints)
            .map_err(|e| DomainError::Internal(format!("failed to serialize endpoints: {e}")))?;

        let row = sqlx::query_as::<_, WorkloadRow>(
            r#"
            INSERT INTO discovered_workloads (
                id, provider_type, namespace, workload_name, engine, detected_version, endpoints, status, details, last_seen_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            ON CONFLICT (provider_type, COALESCE(namespace, ''), workload_name) DO UPDATE
            SET detected_version = EXCLUDED.detected_version,
                endpoints = EXCLUDED.endpoints,
                status = EXCLUDED.status,
                details = EXCLUDED.details,
                last_seen_at = NOW()
            RETURNING id, provider_type, namespace, workload_name, engine, detected_version, endpoints, status, details, last_seen_at
            "#,
        )
        .bind(id)
        .bind(workload.provider_type.as_str())
        .bind(&workload.namespace)
        .bind(&workload.workload_name)
        .bind(workload.engine.as_str())
        .bind(&workload.detected_version)
        .bind(endpoints_json)
        .bind(&workload.status)
        .bind(&workload.details)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "record discovered workload"))?;

        row.try_into()
    }

    async fn list_discovered_workloads(
        &self,
        provider_type: Option<ProviderType>,
    ) -> Result<Vec<DiscoveredWorkload>, DomainError> {
        let rows = if let Some(pt) = provider_type {
            sqlx::query_as::<_, WorkloadRow>(
                r#"
                SELECT id, provider_type, namespace, workload_name, engine, detected_version, endpoints, status, details, last_seen_at
                FROM discovered_workloads
                WHERE provider_type = $1
                ORDER BY last_seen_at DESC
                "#,
            )
            .bind(pt.as_str())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_sqlx_error(e, "list discovered workloads by provider"))?
        } else {
            sqlx::query_as::<_, WorkloadRow>(
                r#"
                SELECT id, provider_type, namespace, workload_name, engine, detected_version, endpoints, status, details, last_seen_at
                FROM discovered_workloads
                ORDER BY last_seen_at DESC
                LIMIT 100
                "#,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_sqlx_error(e, "list discovered workloads"))?
        };

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn get_discovered_workload_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<DiscoveredWorkload>, DomainError> {
        let row = sqlx::query_as::<_, WorkloadRow>(
            r#"
            SELECT id, provider_type, namespace, workload_name, engine, detected_version, endpoints, status, details, last_seen_at
            FROM discovered_workloads
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get discovered workload by id"))?;

        row.map(TryInto::try_into).transpose()
    }
}
