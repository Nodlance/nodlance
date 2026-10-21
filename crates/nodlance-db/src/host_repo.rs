//! PostgreSQL implementation of HostRepository.

use chrono::{DateTime, Utc};
use nodlance_domain::{
    DomainError, Host, HostFilter, HostRepository, HostStatus, NewHost, UpdateHost,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgHostRepository {
    pool: PgPool,
}

impl PgHostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct HostRow {
    id: Uuid,
    hostname: String,
    ip_address: Option<String>,
    os_family: Option<String>,
    os_version: Option<String>,
    labels: serde_json::Value,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Uuid,
}

impl TryFrom<HostRow> for Host {
    type Error = DomainError;

    fn try_from(row: HostRow) -> Result<Self, Self::Error> {
        let status = row
            .status
            .parse::<HostStatus>()
            .map_err(|err| DomainError::Internal(format!("invalid host status in db: {err}")))?;

        Ok(Self {
            id: row.id,
            hostname: row.hostname,
            ip_address: row.ip_address,
            os_family: row.os_family,
            os_version: row.os_version,
            labels: row.labels,
            status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
        })
    }
}

impl HostRepository for PgHostRepository {
    async fn create(&self, created_by: Uuid, new_host: &NewHost) -> Result<Host, DomainError> {
        let id = Uuid::now_v7();
        let labels = new_host
            .labels
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));

        let row = sqlx::query_as::<_, HostRow>(
            r#"
            INSERT INTO hosts (id, hostname, ip_address, os_family, os_version, labels, status, created_at, updated_at, created_by)
            VALUES ($1, $2, $3::INET, $4, $5, $6, 'active', now(), now(), $7)
            RETURNING id, hostname, ip_address::TEXT as ip_address, os_family, os_version, labels, status, created_at, updated_at, created_by
            "#,
        )
        .bind(id)
        .bind(&new_host.hostname)
        .bind(new_host.ip_address.as_deref())
        .bind(new_host.os_family.as_deref())
        .bind(new_host.os_version.as_deref())
        .bind(labels)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "create host"))?;

        row.try_into()
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Host>, DomainError> {
        let row = sqlx::query_as::<_, HostRow>(
            r#"
            SELECT id, hostname, ip_address::TEXT as ip_address, os_family, os_version, labels, status, created_at, updated_at, created_by
            FROM hosts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get host by id"))?;

        row.map(Host::try_from).transpose()
    }

    async fn list(&self, filter: &HostFilter) -> Result<Vec<Host>, DomainError> {
        let status_str = filter.status.map(|s| s.as_str());
        let limit = filter.limit.unwrap_or(50).clamp(1, 100);
        let offset = filter.offset.unwrap_or(0).max(0);

        let rows = sqlx::query_as::<_, HostRow>(
            r#"
            SELECT id, hostname, ip_address::TEXT as ip_address, os_family, os_version, labels, status, created_at, updated_at, created_by
            FROM hosts
            WHERE ($1::TEXT IS NULL OR status = $1)
              AND ($2::TEXT IS NULL OR hostname ILIKE '%' || $2 || '%')
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(status_str)
        .bind(filter.hostname_contains.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list hosts"))?;

        rows.into_iter().map(Host::try_from).collect()
    }

    async fn update(&self, id: Uuid, update: &UpdateHost) -> Result<Host, DomainError> {
        let existing = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("host not found: {id}")))?;

        let hostname = update.hostname.as_deref().unwrap_or(&existing.hostname);
        let ip_address = update
            .ip_address
            .as_deref()
            .or(existing.ip_address.as_deref());
        let os_family = update
            .os_family
            .as_deref()
            .or(existing.os_family.as_deref());
        let os_version = update
            .os_version
            .as_deref()
            .or(existing.os_version.as_deref());
        let labels = update.labels.as_ref().unwrap_or(&existing.labels);
        let status = update
            .status
            .map(|s| s.as_str())
            .unwrap_or_else(|| existing.status.as_str());

        let row = sqlx::query_as::<_, HostRow>(
            r#"
            UPDATE hosts
            SET hostname = $2, ip_address = $3::INET, os_family = $4, os_version = $5, labels = $6, status = $7, updated_at = now()
            WHERE id = $1
            RETURNING id, hostname, ip_address::TEXT as ip_address, os_family, os_version, labels, status, created_at, updated_at, created_by
            "#,
        )
        .bind(id)
        .bind(hostname)
        .bind(ip_address)
        .bind(os_family)
        .bind(os_version)
        .bind(labels)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "update host"))?;

        row.try_into()
    }

    async fn decommission(&self, id: Uuid) -> Result<Host, DomainError> {
        let row = sqlx::query_as::<_, HostRow>(
            r#"
            UPDATE hosts
            SET status = 'decommissioned', updated_at = now()
            WHERE id = $1
            RETURNING id, hostname, ip_address::TEXT as ip_address, os_family, os_version, labels, status, created_at, updated_at, created_by
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "decommission host"))?
        .ok_or_else(|| DomainError::NotFound(format!("host not found: {id}")))?;

        row.try_into()
    }
}
