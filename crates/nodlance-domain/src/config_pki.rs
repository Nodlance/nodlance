//! Domain models for centralized database configuration management (rolling diff & reload)
//! and built-in X.509 Certificate Authority (mTLS PKI).

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

/// Type of X.509 certificate in the Nodlance PKI hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertType {
    RootCa,
    Server,
    Client,
}

impl CertType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RootCa => "root_ca",
            Self::Server => "server",
            Self::Client => "client",
        }
    }
}

impl fmt::Display for CertType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CertType {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "root_ca" | "ca" => Ok(Self::RootCa),
            "server" | "node" => Ok(Self::Server),
            "client" | "app" => Ok(Self::Client),
            other => Err(DomainError::Validation(format!(
                "invalid cert type: '{other}'"
            ))),
        }
    }
}

/// A stored X.509 Certificate record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PkiCertificate {
    pub id: Uuid,
    pub cluster_id: Option<Uuid>,
    pub common_name: String,
    pub cert_type: CertType,
    pub certificate_pem: String,
    pub fingerprint_sha256: String,
    pub valid_from: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub created_at: DateTime<Utc>,
}

impl PkiCertificate {
    pub fn days_until_expiration(&self) -> i64 {
        let diff = self.expires_at.signed_duration_since(Utc::now());
        diff.num_days()
    }

    pub fn is_nearing_expiration(&self, threshold_days: i64) -> bool {
        self.days_until_expiration() <= threshold_days
    }
}

/// An issued certificate bundle containing the public certificate and private key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedCertificate {
    pub certificate: PkiCertificate,
    pub private_key_pem: String,
}

/// A centralized database configuration template (e.g. `postgresql.conf`, `my.cnf`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub id: Uuid,
    pub cluster_id: Option<Uuid>,
    pub name: String,
    pub engine: String, // 'postgres', 'mysql', 'galera'
    pub file_path: String,
    pub content: String,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConfigTemplate {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() {
            return Err(DomainError::Validation(
                "template name cannot be empty".to_string(),
            ));
        }
        if self.file_path.trim().is_empty() {
            return Err(DomainError::Validation(
                "file_path cannot be empty".to_string(),
            ));
        }
        if self.content.trim().is_empty() {
            return Err(DomainError::Validation(
                "content cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

/// Parameter diff entry comparing on-disk or runtime value against template target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigDiff {
    pub parameter: String,
    pub current_value: String,
    pub target_value: String,
    pub requires_restart: bool,
}

/// Checks whether changing a specific parameter requires a full daemon restart.
pub fn parameter_requires_restart(engine: &str, param: &str) -> bool {
    let lower_param = param.to_lowercase();
    match engine.to_lowercase().as_str() {
        "postgres" | "postgresql" => matches!(
            lower_param.as_str(),
            "shared_buffers"
                | "max_connections"
                | "port"
                | "listen_addresses"
                | "archive_mode"
                | "wal_level"
                | "max_worker_processes"
                | "max_parallel_workers"
        ),
        "mysql" | "mariadb" | "galera" => matches!(
            lower_param.as_str(),
            "innodb_buffer_pool_size"
                | "port"
                | "bind-address"
                | "datadir"
                | "socket"
                | "log_bin"
                | "server_id"
                | "wsrep_provider"
        ),
        _ => false,
    }
}

/// Parses key-value configurations into a map.
fn parse_config_lines(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        // Handle key = value or key: value
        if let Some((k, v)) = trimmed.split_once('=').or_else(|| trimmed.split_once(':')) {
            let key = k.trim().to_string();
            let val = v.trim().trim_matches('\'').trim_matches('"').to_string();
            map.insert(key, val);
        }
    }
    map
}

/// Calculates diff entries between current configuration and target template.
pub fn calculate_config_diff(
    engine: &str,
    current_content: &str,
    target_content: &str,
) -> Vec<ConfigDiff> {
    let current_map = parse_config_lines(current_content);
    let target_map = parse_config_lines(target_content);

    let mut diffs = Vec::new();

    for (k, target_v) in target_map {
        let current_v = current_map
            .get(&k)
            .cloned()
            .unwrap_or_else(|| "<unset>".to_string());
        if current_v != target_v {
            let requires_restart = parameter_requires_restart(engine, &k);
            diffs.push(ConfigDiff {
                parameter: k,
                current_value: current_v,
                target_value: target_v,
                requires_restart,
            });
        }
    }

    diffs
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_cert_type_parsing() {
        assert_eq!(CertType::from_str("root_ca").unwrap(), CertType::RootCa);
        assert_eq!(CertType::from_str("server").unwrap(), CertType::Server);
        assert_eq!(CertType::from_str("client").unwrap(), CertType::Client);
        assert!(CertType::from_str("invalid").is_err());
    }

    #[test]
    fn test_certificate_expiration_check() {
        let cert = PkiCertificate {
            id: Uuid::now_v7(),
            cluster_id: Some(Uuid::now_v7()),
            common_name: "db-node-01.internal".to_string(),
            cert_type: CertType::Server,
            certificate_pem: "-----BEGIN CERTIFICATE-----\n...".to_string(),
            fingerprint_sha256: "ab812f8e...".to_string(),
            valid_from: Utc::now() - chrono::Duration::days(300),
            expires_at: Utc::now() + chrono::Duration::days(15),
            is_revoked: false,
            created_at: Utc::now(),
        };

        assert!(cert.is_nearing_expiration(30));
        assert!(!cert.is_nearing_expiration(10));
    }

    #[test]
    fn test_config_diff_calculation() {
        let current = r#"
        # PostgreSQL config
        shared_buffers = 4GB
        work_mem = 64MB
        max_connections = 200
        "#;

        let target = r#"
        shared_buffers = 8GB
        work_mem = 128MB
        max_connections = 200
        maintenance_work_mem = 1GB
        "#;

        let diffs = calculate_config_diff("postgres", current, target);
        assert_eq!(diffs.len(), 3);

        let sb_diff = diffs
            .iter()
            .find(|d| d.parameter == "shared_buffers")
            .unwrap();
        assert_eq!(sb_diff.current_value, "4GB");
        assert_eq!(sb_diff.target_value, "8GB");
        assert!(sb_diff.requires_restart);

        let wm_diff = diffs.iter().find(|d| d.parameter == "work_mem").unwrap();
        assert_eq!(wm_diff.current_value, "64MB");
        assert_eq!(wm_diff.target_value, "128MB");
        assert!(!wm_diff.requires_restart);
    }
}
