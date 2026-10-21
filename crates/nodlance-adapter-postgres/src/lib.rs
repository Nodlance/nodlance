//! PostgreSQL engine health, replication, and role observation adapter.

use std::time::Duration;

use nodlance_domain::{NodeHealth, NodeRole};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, postgres::PgPoolOptions};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PostgresAdapterError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("probe timeout: {0}")]
    Timeout(String),
}

/// Structured observation report resulting from a database engine probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresObservationReport {
    pub role: NodeRole,
    pub health: NodeHealth,
    pub replication_lag_seconds: Option<f64>,
    pub is_read_only: bool,
    pub details: serde_json::Value,
}

/// Probes a PostgreSQL database endpoint to observe its role, health, and replication status.
pub async fn probe_postgres(
    connection_url: &str,
    timeout: Duration,
) -> Result<PostgresObservationReport, PostgresAdapterError> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(timeout)
        .connect(connection_url)
        .await
        .map_err(|e| PostgresAdapterError::Connection(e.to_string()))?;

    // 1. Check recovery status: true means standby/replica, false means primary
    let is_in_recovery: bool = sqlx::query_scalar("SELECT pg_is_in_recovery()")
        .fetch_one(&pool)
        .await
        .map_err(|e| PostgresAdapterError::Query(format!("pg_is_in_recovery check failed: {e}")))?;

    // 2. Fetch server version
    let version: String = sqlx::query_scalar("SHOW server_version")
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|_| "unknown".to_string());

    let (role, is_read_only, lag, details) = if is_in_recovery {
        // Replica observation
        #[derive(FromRow)]
        struct LagRow {
            lag_seconds: Option<f64>,
        }

        let lag_query = r#"
            SELECT EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp())) AS lag_seconds
        "#;
        let lag_row = sqlx::query_as::<_, LagRow>(lag_query)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

        let lag = lag_row.and_then(|r| r.lag_seconds);

        (
            NodeRole::Replica,
            true,
            lag,
            serde_json::json!({
                "server_version": version,
                "in_recovery": true,
                "replay_lag_seconds": lag,
            }),
        )
    } else {
        // Primary observation: check active streaming replicas
        let active_replicas: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_stat_replication")
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

        (
            NodeRole::Primary,
            false,
            None,
            serde_json::json!({
                "server_version": version,
                "in_recovery": false,
                "active_streaming_replicas": active_replicas,
            }),
        )
    };

    Ok(PostgresObservationReport {
        role,
        health: NodeHealth::Healthy,
        replication_lag_seconds: lag,
        is_read_only,
        details,
    })
}
