//! PostgreSQL implementation of AuditRepository.

use chrono::{DateTime, Utc};
use nodlance_domain::{AuditEvent, AuditFilter, AuditRepository, DomainError, NewAuditEvent};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgAuditRepository {
    pool: PgPool,
}

impl PgAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct AuditRow {
    id: Uuid,
    timestamp: DateTime<Utc>,
    actor_id: Uuid,
    action: String,
    resource_type: String,
    resource_id: Option<Uuid>,
    details: Option<serde_json::Value>,
    source_ip: Option<String>,
}

impl From<AuditRow> for AuditEvent {
    fn from(row: AuditRow) -> Self {
        Self {
            id: row.id,
            timestamp: row.timestamp,
            actor_id: row.actor_id,
            action: row.action,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            details: row.details,
            source_ip: row.source_ip,
        }
    }
}

impl AuditRepository for PgAuditRepository {
    async fn record(&self, event: &NewAuditEvent) -> Result<AuditEvent, DomainError> {
        let id = Uuid::now_v7();

        let row = sqlx::query_as::<_, AuditRow>(
            r#"
            INSERT INTO audit_events (id, timestamp, actor_id, action, resource_type, resource_id, details, source_ip)
            VALUES ($1, now(), $2, $3, $4, $5, $6, $7::INET)
            RETURNING id, timestamp, actor_id, action, resource_type, resource_id, details, source_ip::TEXT as source_ip
            "#,
        )
        .bind(id)
        .bind(event.actor_id)
        .bind(&event.action)
        .bind(&event.resource_type)
        .bind(event.resource_id)
        .bind(event.details.as_ref())
        .bind(event.source_ip.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "record audit event"))?;

        Ok(row.into())
    }

    async fn list(&self, filter: &AuditFilter) -> Result<Vec<AuditEvent>, DomainError> {
        let limit = filter.limit.unwrap_or(50).clamp(1, 100);
        let offset = filter.offset.unwrap_or(0).max(0);

        let rows = sqlx::query_as::<_, AuditRow>(
            r#"
            SELECT id, timestamp, actor_id, action, resource_type, resource_id, details, source_ip::TEXT as source_ip
            FROM audit_events
            WHERE ($1::UUID IS NULL OR actor_id = $1)
              AND ($2::TEXT IS NULL OR action = $2)
              AND ($3::TEXT IS NULL OR resource_type = $3)
              AND ($4::UUID IS NULL OR resource_id = $4)
              AND ($5::TIMESTAMPTZ IS NULL OR timestamp >= $5)
              AND ($6::TIMESTAMPTZ IS NULL OR timestamp <= $6)
            ORDER BY timestamp DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(filter.actor_id)
        .bind(filter.action.as_deref())
        .bind(filter.resource_type.as_deref())
        .bind(filter.resource_id)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list audit events"))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
