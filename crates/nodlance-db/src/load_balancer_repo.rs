//! PostgreSQL implementation of LoadBalancerRepository.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use nodlance_domain::{
    DomainError, LoadBalancer, LoadBalancerConfigSnapshot, LoadBalancerFilter,
    LoadBalancerRepository, LoadBalancerStatus, LoadBalancerType, NewLoadBalancer,
    UpdateLoadBalancer,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgLoadBalancerRepository {
    pool: PgPool,
}

impl PgLoadBalancerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct LoadBalancerRow {
    id: Uuid,
    cluster_id: Uuid,
    host_id: Uuid,
    lb_type: String,
    listen_port: i32,
    admin_port: Option<i32>,
    status: String,
    config_snapshot: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<LoadBalancerRow> for LoadBalancer {
    type Error = DomainError;

    fn try_from(row: LoadBalancerRow) -> Result<Self, Self::Error> {
        let lb_type = LoadBalancerType::from_str(&row.lb_type)
            .map_err(|e| DomainError::Internal(format!("corrupt lb_type in db: {e}")))?;

        let status = LoadBalancerStatus::from_str(&row.status)
            .map_err(|e| DomainError::Internal(format!("corrupt status in db: {e}")))?;

        let config_snapshot: LoadBalancerConfigSnapshot =
            serde_json::from_value(row.config_snapshot).map_err(|e| {
                DomainError::Internal(format!("corrupt config_snapshot json in db: {e}"))
            })?;

        Ok(Self {
            id: row.id,
            cluster_id: row.cluster_id,
            host_id: row.host_id,
            lb_type,
            listen_port: row.listen_port as u16,
            admin_port: row.admin_port.map(|p| p as u16),
            status,
            config_snapshot,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl LoadBalancerRepository for PgLoadBalancerRepository {
    async fn create(&self, new_lb: &NewLoadBalancer) -> Result<LoadBalancer, DomainError> {
        new_lb.validate()?;

        let id = Uuid::now_v7();
        let lb_type_str = new_lb.lb_type.as_str();
        let status_str = LoadBalancerStatus::Active.as_str();
        let config_json = serde_json::to_value(&new_lb.config_snapshot).map_err(|e| {
            DomainError::Internal(format!("failed to serialize config_snapshot: {e}"))
        })?;
        let listen_port = new_lb.listen_port as i32;
        let admin_port = new_lb.admin_port.map(|p| p as i32);

        let row = sqlx::query_as::<_, LoadBalancerRow>(
            r#"
            INSERT INTO load_balancers (
                id, cluster_id, host_id, lb_type, listen_port, admin_port, status, config_snapshot, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING id, cluster_id, host_id, lb_type, listen_port, admin_port, status, config_snapshot, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(new_lb.cluster_id)
        .bind(new_lb.host_id)
        .bind(lb_type_str)
        .bind(listen_port)
        .bind(admin_port)
        .bind(status_str)
        .bind(config_json)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "create load balancer"))?;

        LoadBalancer::try_from(row)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<LoadBalancer>, DomainError> {
        let row = sqlx::query_as::<_, LoadBalancerRow>(
            r#"
            SELECT id, cluster_id, host_id, lb_type, listen_port, admin_port, status, config_snapshot, created_at, updated_at
            FROM load_balancers
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get load balancer by id"))?;

        match row {
            Some(r) => LoadBalancer::try_from(r).map(Some),
            None => Ok(None),
        }
    }

    async fn list(&self, filter: &LoadBalancerFilter) -> Result<Vec<LoadBalancer>, DomainError> {
        let rows = sqlx::query_as::<_, LoadBalancerRow>(
            r#"
            SELECT id, cluster_id, host_id, lb_type, listen_port, admin_port, status, config_snapshot, created_at, updated_at
            FROM load_balancers
            WHERE ($1::uuid IS NULL OR cluster_id = $1)
              AND ($2::uuid IS NULL OR host_id = $2)
              AND ($3::text IS NULL OR lb_type = $3)
              AND ($4::text IS NULL OR status = $4)
            ORDER BY created_at DESC
            "#,
        )
        .bind(filter.cluster_id)
        .bind(filter.host_id)
        .bind(filter.lb_type.map(|t| t.as_str()))
        .bind(filter.status.map(|s| s.as_str()))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list load balancers"))?;

        rows.into_iter().map(LoadBalancer::try_from).collect()
    }

    async fn update(
        &self,
        id: Uuid,
        update: &UpdateLoadBalancer,
    ) -> Result<LoadBalancer, DomainError> {
        let status_str = update.status.map(|s| s.as_str());
        let config_json = update
            .config_snapshot
            .as_ref()
            .map(|c| {
                serde_json::to_value(c).map_err(|e| {
                    DomainError::Internal(format!("failed to serialize config_snapshot: {e}"))
                })
            })
            .transpose()?;

        let row = sqlx::query_as::<_, LoadBalancerRow>(
            r#"
            UPDATE load_balancers
            SET status = COALESCE($2, status),
                config_snapshot = COALESCE($3, config_snapshot),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, cluster_id, host_id, lb_type, listen_port, admin_port, status, config_snapshot, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(status_str)
        .bind(config_json)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "update load balancer"))?
        .ok_or_else(|| DomainError::NotFound(format!("load balancer '{id}' not found")))?;

        LoadBalancer::try_from(row)
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            DELETE FROM load_balancers
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "delete load balancer"))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!(
                "load balancer '{id}' not found"
            )));
        }

        Ok(())
    }
}
