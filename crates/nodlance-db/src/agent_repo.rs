//! PostgreSQL implementation of AgentRepository.

use chrono::{DateTime, Utc};
use nodlance_domain::{AgentIdentity, AgentRepository, AgentStatus, DomainError};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgAgentRepository {
    pool: PgPool,
}

impl PgAgentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct AgentRow {
    id: Uuid,
    host_id: Uuid,
    fingerprint: String,
    status: String,
    enrolled_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
    revoked_by: Option<Uuid>,
    created_at: DateTime<Utc>,
}

impl TryFrom<AgentRow> for AgentIdentity {
    type Error = DomainError;

    fn try_from(row: AgentRow) -> Result<Self, Self::Error> {
        let status = row
            .status
            .parse::<AgentStatus>()
            .map_err(|err| DomainError::Internal(format!("invalid agent status in db: {err}")))?;

        Ok(Self {
            id: row.id,
            host_id: row.host_id,
            fingerprint: row.fingerprint,
            status,
            enrolled_at: row.enrolled_at,
            revoked_at: row.revoked_at,
            revoked_by: row.revoked_by,
            created_at: row.created_at,
        })
    }
}

impl AgentRepository for PgAgentRepository {
    async fn enroll(&self, host_id: Uuid, fingerprint: &str) -> Result<AgentIdentity, DomainError> {
        let id = Uuid::now_v7();

        // First revoke any previous enrolled identities for this host
        let _ = sqlx::query(
            r#"
            UPDATE agent_identities
            SET status = 'revoked', revoked_at = now()
            WHERE host_id = $1 AND status != 'revoked'
            "#,
        )
        .bind(host_id)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "revoke previous agents"))?;

        let row = sqlx::query_as::<_, AgentRow>(
            r#"
            INSERT INTO agent_identities (id, host_id, fingerprint, status, enrolled_at, created_at)
            VALUES ($1, $2, $3, 'enrolled', now(), now())
            RETURNING id, host_id, fingerprint, status, enrolled_at, revoked_at, revoked_by, created_at
            "#,
        )
        .bind(id)
        .bind(host_id)
        .bind(fingerprint)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "enroll agent"))?;

        row.try_into()
    }

    async fn get_by_host_id(&self, host_id: Uuid) -> Result<Option<AgentIdentity>, DomainError> {
        let row = sqlx::query_as::<_, AgentRow>(
            r#"
            SELECT id, host_id, fingerprint, status, enrolled_at, revoked_at, revoked_by, created_at
            FROM agent_identities
            WHERE host_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(host_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get agent by host_id"))?;

        row.map(AgentIdentity::try_from).transpose()
    }

    async fn get_by_fingerprint(
        &self,
        fingerprint: &str,
    ) -> Result<Option<AgentIdentity>, DomainError> {
        let row = sqlx::query_as::<_, AgentRow>(
            r#"
            SELECT id, host_id, fingerprint, status, enrolled_at, revoked_at, revoked_by, created_at
            FROM agent_identities
            WHERE fingerprint = $1
            "#,
        )
        .bind(fingerprint)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get agent by fingerprint"))?;

        row.map(AgentIdentity::try_from).transpose()
    }

    async fn revoke(&self, host_id: Uuid, revoked_by: Uuid) -> Result<AgentIdentity, DomainError> {
        let row = sqlx::query_as::<_, AgentRow>(
            r#"
            UPDATE agent_identities
            SET status = 'revoked', revoked_at = now(), revoked_by = $2
            WHERE id = (
                SELECT id FROM agent_identities
                WHERE host_id = $1 AND status != 'revoked'
                ORDER BY created_at DESC
                LIMIT 1
            )
            RETURNING id, host_id, fingerprint, status, enrolled_at, revoked_at, revoked_by, created_at
            "#,
        )
        .bind(host_id)
        .bind(revoked_by)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "revoke agent"))?
        .ok_or_else(|| {
            DomainError::NotFound(format!("no active agent found to revoke for host {host_id}"))
        })?;

        row.try_into()
    }
}
