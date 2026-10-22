//! Domain models for MySQL/MariaDB Galera Cluster & Galera Arbitrator (garbd).
//!
//! Covers Severalnines ClusterControl parity for multi-primary replication,
//! SST/IST state observation, and split-brain arbitration.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

/// Galera wsrep local state tracking internal node state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WsrepLocalState {
    Joining,
    Donor,
    Joined,
    Synced,
    Error,
}

impl WsrepLocalState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Joining => "joining",
            Self::Donor => "donor",
            Self::Joined => "joined",
            Self::Synced => "synced",
            Self::Error => "error",
        }
    }

    pub fn from_state_code(code: u32) -> Self {
        match code {
            1 => Self::Joining,
            2 => Self::Donor,
            3 => Self::Joined,
            4 => Self::Synced,
            _ => Self::Error,
        }
    }
}

impl fmt::Display for WsrepLocalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for WsrepLocalState {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "joining" => Ok(Self::Joining),
            "donor" | "desync" => Ok(Self::Donor),
            "joined" => Ok(Self::Joined),
            "synced" => Ok(Self::Synced),
            "error" | "closed" => Ok(Self::Error),
            other => Err(DomainError::Validation(format!(
                "unknown wsrep local state: '{other}'"
            ))),
        }
    }
}

/// Cluster-wide view status indicating whether this partition has primary quorum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WsrepClusterStatus {
    Primary,
    NonPrimary,
}

impl WsrepClusterStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::NonPrimary => "non_primary",
        }
    }

    pub fn has_quorum(self) -> bool {
        matches!(self, Self::Primary)
    }
}

impl fmt::Display for WsrepClusterStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for WsrepClusterStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "primary" => Ok(Self::Primary),
            "non-primary" | "non_primary" => Ok(Self::NonPrimary),
            other => Err(DomainError::Validation(format!(
                "unknown wsrep cluster status: '{other}'"
            ))),
        }
    }
}

/// Supported State Snapshot Transfer (SST) methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SstMethod {
    Mariabackup,
    XtrabackupV2,
    Rsync,
}

impl SstMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mariabackup => "mariabackup",
            Self::XtrabackupV2 => "xtrabackup-v2",
            Self::Rsync => "rsync",
        }
    }
}

impl fmt::Display for SstMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SstMethod {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mariabackup" => Ok(Self::Mariabackup),
            "xtrabackup-v2" | "xtrabackup" => Ok(Self::XtrabackupV2),
            "rsync" => Ok(Self::Rsync),
            other => Err(DomainError::Validation(format!(
                "unknown sst method: '{other}'"
            ))),
        }
    }
}

/// Galera Arbitrator (`garbd`) configuration payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GaleraArbitratorConfig {
    pub cluster_name: String,
    pub cluster_address: String, // gcomm://node1:4567,node2:4567
    pub listen_port: u16,         // default 4567
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sst_donor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<String>,
}

impl GaleraArbitratorConfig {
    /// Generates standard /etc/default/garbd configuration file.
    pub fn generate_garbd_env_file(&self) -> String {
        let mut out = String::new();
        out.push_str("# Nodlance Generated Galera Arbitrator (garbd) configuration\n");
        out.push_str(&format!("GALERA_GROUP=\"{}\"\n", self.cluster_name));
        out.push_str(&format!("GALERA_NODES=\"{}\"\n", self.cluster_address));
        out.push_str(&format!("GALERA_PORT=\"{}\"\n", self.listen_port));
        if let Some(donor) = &self.sst_donor {
            out.push_str(&format!("GALERA_DONOR=\"{}\"\n", donor));
        }
        if let Some(opts) = &self.options {
            out.push_str(&format!("GALERA_OPTIONS=\"{}\"\n", opts));
        } else {
            out.push_str("GALERA_OPTIONS=\"base_port=4567;gcache.size=0\"\n");
        }
        out.push_str("LOG_FILE=\"/var/log/garbd.log\"\n");
        out
    }

    /// Generates systemd service unit file for garbd daemon.
    pub fn generate_systemd_unit(&self) -> String {
        let mut unit = String::new();
        unit.push_str("[Unit]\nDescription=Galera Arbitrator Daemon (garbd)\nAfter=network.target\n\n");
        unit.push_str("[Service]\nType=simple\nUser=nobody\nGroup=nogroup\nEnvironmentFile=-/etc/default/garbd\n");
        unit.push_str("ExecStart=/usr/bin/garbd -a ${GALERA_NODES} -g ${GALERA_GROUP} -o ${GALERA_OPTIONS} -l ${LOG_FILE}\n");
        unit.push_str("Restart=always\nRestartSec=5\n\n[Install]\nWantedBy=multi-user.target\n");
        unit
    }
}

/// Real-time Galera observation telemetry payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GaleraObservation {
    pub cluster_id: Uuid,
    pub node_id: Uuid,
    pub cluster_state_uuid: String,
    pub cluster_size: u32,
    pub cluster_status: WsrepClusterStatus,
    pub local_state: WsrepLocalState,
    pub local_state_comment: String,
    pub ready: bool,
    pub connected: bool,
    pub flow_control_paused_ns: u64,
    pub last_committed_seqno: i64,
    pub recorded_at: DateTime<Utc>,
}

impl GaleraObservation {
    /// Detects whether this node is in an operational ready state capable of serving queries.
    pub fn is_operational(&self) -> bool {
        self.connected && self.ready && self.local_state == WsrepLocalState::Synced && self.cluster_status == WsrepClusterStatus::Primary
    }

    /// Evaluates if cluster partition is degraded or at risk of split-brain.
    pub fn evaluate_quorum_health(&self) -> Result<(), DomainError> {
        if self.cluster_status != WsrepClusterStatus::Primary {
            return Err(DomainError::Validation(format!(
                "Galera cluster partition is non-primary! Quorum lost (wsrep_cluster_status = {})",
                self.cluster_status
            )));
        }

        if !self.ready {
            return Err(DomainError::Validation(
                "Galera node is not ready to accept transactions".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wsrep_states_and_status() {
        assert_eq!(WsrepLocalState::from_state_code(1), WsrepLocalState::Joining);
        assert_eq!(WsrepLocalState::from_state_code(2), WsrepLocalState::Donor);
        assert_eq!(WsrepLocalState::from_state_code(3), WsrepLocalState::Joined);
        assert_eq!(WsrepLocalState::from_state_code(4), WsrepLocalState::Synced);
        assert_eq!(WsrepLocalState::from_state_code(99), WsrepLocalState::Error);

        assert_eq!(WsrepLocalState::from_str("synced").unwrap(), WsrepLocalState::Synced);
        assert_eq!(WsrepClusterStatus::from_str("primary").unwrap(), WsrepClusterStatus::Primary);
        assert_eq!(WsrepClusterStatus::from_str("non-primary").unwrap(), WsrepClusterStatus::NonPrimary);
    }

    #[test]
    fn test_garbd_config_and_systemd_generation() {
        let garbd = GaleraArbitratorConfig {
            cluster_name: "prod_galera".to_string(),
            cluster_address: "gcomm://10.0.0.1:4567,10.0.0.2:4567".to_string(),
            listen_port: 4567,
            sst_donor: Some("node1".to_string()),
            options: None,
        };

        let env_file = garbd.generate_garbd_env_file();
        assert!(env_file.contains("GALERA_GROUP=\"prod_galera\""));
        assert!(env_file.contains("GALERA_NODES=\"gcomm://10.0.0.1:4567,10.0.0.2:4567\""));
        assert!(env_file.contains("GALERA_DONOR=\"node1\""));

        let unit = garbd.generate_systemd_unit();
        assert!(unit.contains("ExecStart=/usr/bin/garbd"));
    }

    #[test]
    fn test_galera_observation_operational_check() {
        let obs = GaleraObservation {
            cluster_id: Uuid::now_v7(),
            node_id: Uuid::now_v7(),
            cluster_state_uuid: "c3894474-9548-11ec-a63e-2fa31fc48479".to_string(),
            cluster_size: 3,
            cluster_status: WsrepClusterStatus::Primary,
            local_state: WsrepLocalState::Synced,
            local_state_comment: "Synced".to_string(),
            ready: true,
            connected: true,
            flow_control_paused_ns: 0,
            last_committed_seqno: 123456,
            recorded_at: Utc::now(),
        };

        assert!(obs.is_operational());
        assert!(obs.evaluate_quorum_health().is_ok());

        let broken_obs = GaleraObservation {
            cluster_status: WsrepClusterStatus::NonPrimary,
            ready: false,
            ..obs
        };
        assert!(!broken_obs.is_operational());
        assert!(broken_obs.evaluate_quorum_health().is_err());
    }
}
