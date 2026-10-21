//! PostgreSQL implementation of HeartbeatRepository.

use chrono::{DateTime, Utc};
use nodlance_domain::{AgentHeartbeat, DomainError, HeartbeatRepository, HostMetrics};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgHeartbeatRepository {
    pool: PgPool,
}

impl PgHeartbeatRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct HeartbeatRow {
    id: Uuid,
    host_id: Uuid,
    timestamp: DateTime<Utc>,
    agent_version: String,
    cpu_percent: Option<f32>,
    memory_used_bytes: Option<i64>,
    memory_total_bytes: Option<i64>,
    disk_used_bytes: Option<i64>,
    disk_total_bytes: Option<i64>,
    load_avg_one: Option<f32>,
}

impl From<HeartbeatRow> for AgentHeartbeat {
    fn from(row: HeartbeatRow) -> Self {
        let metrics = match (
            row.cpu_percent,
            row.memory_used_bytes,
            row.memory_total_bytes,
            row.disk_used_bytes,
            row.disk_total_bytes,
            row.load_avg_one,
        ) {
            (
                Some(cpu),
                Some(mem_used),
                Some(mem_total),
                Some(disk_used),
                Some(disk_total),
                Some(load),
            ) => Some(HostMetrics {
                cpu_percent: cpu,
                memory_used_bytes: mem_used as u64,
                memory_total_bytes: mem_total as u64,
                disk_used_bytes: disk_used as u64,
                disk_total_bytes: disk_total as u64,
                load_avg_one: load,
            }),
            _ => None,
        };

        Self {
            id: row.id,
            host_id: row.host_id,
            timestamp: row.timestamp,
            agent_version: row.agent_version,
            metrics,
        }
    }
}

impl HeartbeatRepository for PgHeartbeatRepository {
    async fn record_heartbeat(
        &self,
        host_id: Uuid,
        agent_version: &str,
        metrics: Option<&HostMetrics>,
    ) -> Result<AgentHeartbeat, DomainError> {
        let id = Uuid::now_v7();

        let (cpu, mem_used, mem_total, disk_used, disk_total, load) = match metrics {
            Some(m) => (
                Some(m.cpu_percent),
                Some(m.memory_used_bytes as i64),
                Some(m.memory_total_bytes as i64),
                Some(m.disk_used_bytes as i64),
                Some(m.disk_total_bytes as i64),
                Some(m.load_avg_one),
            ),
            None => (None, None, None, None, None, None),
        };

        let row = sqlx::query_as::<_, HeartbeatRow>(
            r#"
            INSERT INTO agent_heartbeats (id, host_id, timestamp, agent_version, cpu_percent, memory_used_bytes, memory_total_bytes, disk_used_bytes, disk_total_bytes, load_avg_one)
            VALUES ($1, $2, now(), $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, host_id, timestamp, agent_version, cpu_percent, memory_used_bytes, memory_total_bytes, disk_used_bytes, disk_total_bytes, load_avg_one
            "#,
        )
        .bind(id)
        .bind(host_id)
        .bind(agent_version)
        .bind(cpu)
        .bind(mem_used)
        .bind(mem_total)
        .bind(disk_used)
        .bind(disk_total)
        .bind(load)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "record agent heartbeat"))?;

        Ok(row.into())
    }

    async fn get_latest_heartbeat(
        &self,
        host_id: Uuid,
    ) -> Result<Option<AgentHeartbeat>, DomainError> {
        let row = sqlx::query_as::<_, HeartbeatRow>(
            r#"
            SELECT id, host_id, timestamp, agent_version, cpu_percent, memory_used_bytes, memory_total_bytes, disk_used_bytes, disk_total_bytes, load_avg_one
            FROM agent_heartbeats
            WHERE host_id = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(host_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get latest heartbeat"))?;

        Ok(row.map(Into::into))
    }

    async fn get_metrics_history(
        &self,
        host_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AgentHeartbeat>, DomainError> {
        let limit = limit.clamp(1, 200);

        let rows = sqlx::query_as::<_, HeartbeatRow>(
            r#"
            SELECT id, host_id, timestamp, agent_version, cpu_percent, memory_used_bytes, memory_total_bytes, disk_used_bytes, disk_total_bytes, load_avg_one
            FROM agent_heartbeats
            WHERE host_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
        )
        .bind(host_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get metrics history"))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
