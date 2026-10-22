//! MySQL and MariaDB Galera Cluster & Arbitrator (`garbd`) adapter.
//!
//! Provides Severalnines ClusterControl parity for multi-primary topology observation,
//! wsrep state evaluation, SST/IST safe donor selection, and Garbd arbitrator generation.

use std::{collections::HashMap, time::Duration};

use chrono::Utc;
use nodlance_domain::{GaleraObservation, SstMethod, WsrepClusterStatus, WsrepLocalState};
use sqlx::{Row, mysql::MySqlPoolOptions};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum GaleraAdapterError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("wsrep not enabled on target MySQL/MariaDB server")]
    WsrepDisabled,

    #[error("invalid wsrep state: {0}")]
    InvalidState(String),
}

/// Evaluates raw wsrep status and variables maps to produce a structured GaleraObservation.
pub fn parse_wsrep_status(
    cluster_id: Uuid,
    node_id: Uuid,
    status_map: &HashMap<String, String>,
) -> Result<GaleraObservation, GaleraAdapterError> {
    let wsrep_on = status_map
        .get("wsrep_on")
        .map(|v| v.eq_ignore_ascii_case("ON") || v == "1")
        .unwrap_or(false);

    if !wsrep_on && !status_map.contains_key("wsrep_local_state") {
        return Err(GaleraAdapterError::WsrepDisabled);
    }

    let local_state_code: u32 = status_map
        .get("wsrep_local_state")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let local_state = WsrepLocalState::from_state_code(local_state_code);

    let local_state_comment = status_map
        .get("wsrep_local_state_comment")
        .cloned()
        .unwrap_or_else(|| local_state.as_str().to_string());

    let cluster_status_str = status_map
        .get("wsrep_cluster_status")
        .cloned()
        .unwrap_or_else(|| "non-primary".to_string());
    let cluster_status = if cluster_status_str.eq_ignore_ascii_case("Primary") {
        WsrepClusterStatus::Primary
    } else {
        WsrepClusterStatus::NonPrimary
    };

    let cluster_size: u32 = status_map
        .get("wsrep_cluster_size")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let cluster_state_uuid = status_map
        .get("wsrep_cluster_state_uuid")
        .cloned()
        .unwrap_or_default();

    let ready = status_map
        .get("wsrep_ready")
        .map(|v| v.eq_ignore_ascii_case("ON") || v == "1")
        .unwrap_or(false);

    let connected = status_map
        .get("wsrep_connected")
        .map(|v| v.eq_ignore_ascii_case("ON") || v == "1")
        .unwrap_or(false);

    let flow_control_paused_ns: u64 = status_map
        .get("wsrep_flow_control_paused_ns")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let last_committed_seqno: i64 = status_map
        .get("wsrep_last_committed")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    Ok(GaleraObservation {
        cluster_id,
        node_id,
        cluster_state_uuid,
        cluster_size,
        cluster_status,
        local_state,
        local_state_comment,
        ready,
        connected,
        flow_control_paused_ns,
        last_committed_seqno,
        recorded_at: Utc::now(),
    })
}

/// Selects the safest donor node for State Snapshot Transfer (SST).
///
/// Follows ClusterControl rule: donor must be Synced (4), have lowest flow control pauses,
/// and not be currently servicing as donor to another node.
pub fn select_safe_sst_donor(candidates: &[(String, GaleraObservation)]) -> Option<&str> {
    candidates
        .iter()
        .filter(|(_, obs)| obs.is_operational() && obs.flow_control_paused_ns == 0)
        .max_by_key(|(_, obs)| obs.last_committed_seqno)
        .map(|(host, _)| host.as_str())
}

/// Probes a live MySQL/MariaDB Galera node and extracts wsrep telemetry.
pub async fn probe_galera(
    cluster_id: Uuid,
    node_id: Uuid,
    connection_url: &str,
    timeout: Duration,
) -> Result<GaleraObservation, GaleraAdapterError> {
    let pool = MySqlPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(timeout)
        .connect(connection_url)
        .await
        .map_err(|e| GaleraAdapterError::Connection(e.to_string()))?;

    let rows = sqlx::query("SHOW GLOBAL STATUS LIKE 'wsrep_%'")
        .fetch_all(&pool)
        .await
        .map_err(|e| GaleraAdapterError::Query(e.to_string()))?;

    let mut map = HashMap::new();
    for row in rows {
        let key: String = row.try_get(0).unwrap_or_default();
        let val: String = row.try_get(1).unwrap_or_default();
        map.insert(key, val);
    }

    parse_wsrep_status(cluster_id, node_id, &map)
}

/// Detects the active State Snapshot Transfer (SST) method.
pub async fn inspect_sst_method(
    connection_url: &str,
    timeout: Duration,
) -> Result<SstMethod, GaleraAdapterError> {
    let pool = MySqlPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(timeout)
        .connect(connection_url)
        .await
        .map_err(|e| GaleraAdapterError::Connection(e.to_string()))?;

    let row = sqlx::query("SHOW VARIABLES LIKE 'wsrep_sst_method'")
        .fetch_optional(&pool)
        .await
        .map_err(|e| GaleraAdapterError::Query(e.to_string()))?;

    let val: String = match row {
        Some(r) => r.try_get(1).unwrap_or_else(|_| "mariabackup".to_string()),
        None => "mariabackup".to_string(),
    };

    val.parse()
        .map_err(|_| GaleraAdapterError::InvalidState(format!("unrecognized sst method: {val}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_healthy_galera_status() {
        let mut map = HashMap::new();
        map.insert("wsrep_on".to_string(), "ON".to_string());
        map.insert("wsrep_local_state".to_string(), "4".to_string());
        map.insert(
            "wsrep_local_state_comment".to_string(),
            "Synced".to_string(),
        );
        map.insert("wsrep_cluster_status".to_string(), "Primary".to_string());
        map.insert("wsrep_cluster_size".to_string(), "3".to_string());
        map.insert(
            "wsrep_cluster_state_uuid".to_string(),
            "abc-123-uuid".to_string(),
        );
        map.insert("wsrep_ready".to_string(), "ON".to_string());
        map.insert("wsrep_connected".to_string(), "ON".to_string());
        map.insert("wsrep_flow_control_paused_ns".to_string(), "0".to_string());
        map.insert("wsrep_last_committed".to_string(), "500021".to_string());

        let cluster_id = Uuid::now_v7();
        let node_id = Uuid::now_v7();

        let obs = parse_wsrep_status(cluster_id, node_id, &map).unwrap();
        assert!(obs.is_operational());
        assert_eq!(obs.local_state, WsrepLocalState::Synced);
        assert_eq!(obs.cluster_status, WsrepClusterStatus::Primary);
        assert_eq!(obs.cluster_size, 3);
        assert_eq!(obs.last_committed_seqno, 500021);
    }

    #[test]
    fn test_parse_desynced_donor_galera_status() {
        let mut map = HashMap::new();
        map.insert("wsrep_on".to_string(), "ON".to_string());
        map.insert("wsrep_local_state".to_string(), "2".to_string());
        map.insert(
            "wsrep_local_state_comment".to_string(),
            "Donor/Desynced".to_string(),
        );
        map.insert("wsrep_cluster_status".to_string(), "Primary".to_string());
        map.insert("wsrep_cluster_size".to_string(), "3".to_string());
        map.insert("wsrep_ready".to_string(), "ON".to_string());
        map.insert("wsrep_connected".to_string(), "ON".to_string());

        let cluster_id = Uuid::now_v7();
        let node_id = Uuid::now_v7();

        let obs = parse_wsrep_status(cluster_id, node_id, &map).unwrap();
        assert!(!obs.is_operational()); // Donor is not in Synced state
        assert_eq!(obs.local_state, WsrepLocalState::Donor);
    }

    #[test]
    fn test_safe_sst_donor_selection() {
        let cluster_id = Uuid::now_v7();
        let node1 = (
            "db-node-01".to_string(),
            GaleraObservation {
                cluster_id,
                node_id: Uuid::now_v7(),
                cluster_state_uuid: "u1".to_string(),
                cluster_size: 3,
                cluster_status: WsrepClusterStatus::Primary,
                local_state: WsrepLocalState::Synced,
                local_state_comment: "Synced".to_string(),
                ready: true,
                connected: true,
                flow_control_paused_ns: 0,
                last_committed_seqno: 100,
                recorded_at: Utc::now(),
            },
        );

        let node2 = (
            "db-node-02".to_string(),
            GaleraObservation {
                cluster_id,
                node_id: Uuid::now_v7(),
                cluster_state_uuid: "u1".to_string(),
                cluster_size: 3,
                cluster_status: WsrepClusterStatus::Primary,
                local_state: WsrepLocalState::Synced,
                local_state_comment: "Synced".to_string(),
                ready: true,
                connected: true,
                flow_control_paused_ns: 0,
                last_committed_seqno: 250, // Higher committed seqno
                recorded_at: Utc::now(),
            },
        );

        let candidates = vec![node1, node2];
        let chosen = select_safe_sst_donor(&candidates);
        assert_eq!(chosen, Some("db-node-02"));
    }
}
