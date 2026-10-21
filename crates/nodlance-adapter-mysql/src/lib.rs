//! MySQL and MariaDB engine health, replication, and role observation adapter.

use std::time::Duration;

use nodlance_domain::{NodeHealth, NodeRole};
use serde::{Deserialize, Serialize};
use sqlx::{Row, mysql::MySqlPoolOptions};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MySqlAdapterError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("probe timeout: {0}")]
    Timeout(String),
}

/// Structured observation report resulting from a MySQL/MariaDB probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySqlObservationReport {
    pub role: NodeRole,
    pub health: NodeHealth,
    pub replication_lag_seconds: Option<f64>,
    pub is_read_only: bool,
    pub details: serde_json::Value,
}

/// Probes a MySQL or MariaDB database endpoint to observe its role, health, and replication status.
pub async fn probe_mysql(
    connection_url: &str,
    timeout: Duration,
) -> Result<MySqlObservationReport, MySqlAdapterError> {
    let pool = MySqlPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(timeout)
        .connect(connection_url)
        .await
        .map_err(|e| MySqlAdapterError::Connection(e.to_string()))?;

    // 1. Check read_only system variable
    let read_only_val: i32 = sqlx::query_scalar("SELECT @@read_only")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    let is_read_only = read_only_val == 1;

    // 2. Fetch server version
    let version: String = sqlx::query_scalar("SELECT VERSION()")
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|_| "unknown".to_string());

    // 3. Inspect replication status (supports MySQL 8.0.22+ SHOW REPLICA STATUS, falls back to SHOW SLAVE STATUS)
    let replica_status = match sqlx::query("SHOW REPLICA STATUS")
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(row)) => Some(row),
        _ => sqlx::query("SHOW SLAVE STATUS")
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten(),
    };

    let (role, lag) = if let Some(row) = replica_status {
        // Try column "Seconds_Behind_Source", fallback to "Seconds_Behind_Master"
        let seconds_behind: Option<i64> = row
            .try_get("Seconds_Behind_Source")
            .or_else(|_| row.try_get("Seconds_Behind_Master"))
            .ok();

        (NodeRole::Replica, seconds_behind.map(|s| s as f64))
    } else if is_read_only {
        (NodeRole::Replica, None)
    } else {
        (NodeRole::Primary, None)
    };

    Ok(MySqlObservationReport {
        role,
        health: NodeHealth::Healthy,
        replication_lag_seconds: lag,
        is_read_only,
        details: serde_json::json!({
            "server_version": version,
            "read_only": is_read_only,
            "seconds_behind_source": lag,
        }),
    })
}
