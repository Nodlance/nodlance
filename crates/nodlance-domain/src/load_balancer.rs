//! Domain models for database load balancers, connection routers, and Virtual IP (VIP) failover.
//!
//! Covers Severalnines ClusterControl parity for ProxySQL, PgBouncer, HAProxy, and Keepalived.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

/// Type of managed database load balancer or connection router.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancerType {
    ProxySql,
    PgBouncer,
    HaProxy,
    Keepalived,
}

impl LoadBalancerType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProxySql => "proxysql",
            Self::PgBouncer => "pgbouncer",
            Self::HaProxy => "haproxy",
            Self::Keepalived => "keepalived",
        }
    }
}

impl fmt::Display for LoadBalancerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LoadBalancerType {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proxysql" => Ok(Self::ProxySql),
            "pgbouncer" => Ok(Self::PgBouncer),
            "haproxy" => Ok(Self::HaProxy),
            "keepalived" => Ok(Self::Keepalived),
            other => Err(DomainError::Validation(format!(
                "invalid load balancer type: '{other}'"
            ))),
        }
    }
}

/// Operational status of a load balancer instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancerStatus {
    Active,
    Standby,
    Degraded,
    Error,
    Maintenance,
}

impl LoadBalancerStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Standby => "standby",
            Self::Degraded => "degraded",
            Self::Error => "error",
            Self::Maintenance => "maintenance",
        }
    }
}

impl fmt::Display for LoadBalancerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LoadBalancerStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "standby" => Ok(Self::Standby),
            "degraded" => Ok(Self::Degraded),
            "error" => Ok(Self::Error),
            "maintenance" => Ok(Self::Maintenance),
            other => Err(DomainError::Validation(format!(
                "invalid load balancer status: '{other}'"
            ))),
        }
    }
}

/// Connection pooling modes for PostgreSQL PgBouncer instances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PgBouncerPoolMode {
    Session,
    Transaction,
    Statement,
}

impl PgBouncerPoolMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Transaction => "transaction",
            Self::Statement => "statement",
        }
    }
}

impl fmt::Display for PgBouncerPoolMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PgBouncerPoolMode {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "session" => Ok(Self::Session),
            "transaction" => Ok(Self::Transaction),
            "statement" => Ok(Self::Statement),
            other => Err(DomainError::Validation(format!(
                "invalid pgbouncer pool mode: '{other}'"
            ))),
        }
    }
}

/// ProxySQL query routing rule matching SQL patterns to destination hostgroups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxySqlQueryRule {
    pub rule_id: u32,
    pub match_pattern: String,
    pub destination_hostgroup: u32,
    pub active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Backend server registered in ProxySQL hostgroup (writer or reader).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxySqlBackendServer {
    pub hostgroup_id: u32,
    pub hostname: String,
    pub port: u16,
    pub weight: u32,
    pub max_connections: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Structured configuration payload for ProxySQL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxySqlConfig {
    pub admin_username: String,
    pub writer_hostgroup: u32,
    pub reader_hostgroup: u32,
    #[serde(default)]
    pub servers: Vec<ProxySqlBackendServer>,
    #[serde(default)]
    pub query_rules: Vec<ProxySqlQueryRule>,
}

/// Structured configuration payload for PgBouncer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgBouncerConfig {
    pub pool_mode: PgBouncerPoolMode,
    pub max_client_conn: u32,
    pub default_pool_size: u32,
    pub min_pool_size: u32,
    pub reserve_pool_size: u32,
    pub auth_type: String,
    #[serde(default)]
    pub users: Vec<String>,
}

/// Structured configuration payload for HAProxy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaProxyConfig {
    pub check_type: String,
    pub balance_algorithm: String,
    #[serde(default)]
    pub backend_servers: Vec<String>,
    pub timeout_connect_ms: u32,
    pub timeout_client_ms: u32,
    pub timeout_server_ms: u32,
}

/// Structured configuration payload for Keepalived VRRP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeepalivedConfig {
    pub virtual_router_id: u8,
    pub interface: String,
    pub virtual_ip: String,
    pub priority: u8,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_script: Option<String>,
    pub advert_int_seconds: u8,
}

/// Composite JSON-serializable snapshot of load balancer configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadBalancerConfigSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxysql: Option<ProxySqlConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pgbouncer: Option<PgBouncerConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub haproxy: Option<HaProxyConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keepalived: Option<KeepalivedConfig>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub extra: serde_json::Value,
}

/// A managed load balancer or connection router instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadBalancer {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub host_id: Uuid,
    pub lb_type: LoadBalancerType,
    pub listen_port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_port: Option<u16>,
    pub status: LoadBalancerStatus,
    pub config_snapshot: LoadBalancerConfigSnapshot,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LoadBalancer {
    /// Validates the invariants of the load balancer definition.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.listen_port == 0 {
            return Err(DomainError::Validation(
                "listen_port must be greater than 0".to_string(),
            ));
        }

        if let Some(admin) = self.admin_port {
            if admin == self.listen_port {
                return Err(DomainError::Validation(
                    "admin_port cannot be identical to listen_port".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Generates administrative SQLite commands for ProxySQL runtime configuration.
    pub fn generate_proxysql_admin_sql(&self) -> Result<String, DomainError> {
        let config = self.config_snapshot.proxysql.as_ref().ok_or_else(|| {
            DomainError::Validation("ProxySqlConfig missing in config_snapshot".to_string())
        })?;

        let mut sql = String::new();
        sql.push_str("-- Nodlance Generated ProxySQL Configuration\n");
        sql.push_str("DELETE FROM mysql_servers;\n");
        for server in &config.servers {
            sql.push_str(&format!(
                "INSERT INTO mysql_servers (hostgroup_id, hostname, port, weight, max_connections, comment) VALUES ({}, '{}', {}, {}, {}, '{}');\n",
                server.hostgroup_id,
                server.hostname.replace('\'', "''"),
                server.port,
                server.weight,
                server.max_connections,
                server.comment.as_deref().unwrap_or("managed by nodlance").replace('\'', "''")
            ));
        }

        sql.push_str("DELETE FROM mysql_query_rules;\n");
        for rule in &config.query_rules {
            sql.push_str(&format!(
                "INSERT INTO mysql_query_rules (rule_id, match_pattern, destination_hostgroup, active, comment) VALUES ({}, '{}', {}, {}, '{}');\n",
                rule.rule_id,
                rule.match_pattern.replace('\'', "''"),
                rule.destination_hostgroup,
                if rule.active { 1 } else { 0 },
                rule.comment.as_deref().unwrap_or("rule").replace('\'', "''")
            ));
        }

        sql.push_str("LOAD MYSQL SERVERS TO RUNTIME;\nSAVE MYSQL SERVERS TO DISK;\n");
        sql.push_str("LOAD MYSQL QUERY RULES TO RUNTIME;\nSAVE MYSQL QUERY RULES TO DISK;\n");
        Ok(sql)
    }

    /// Generates a valid pgbouncer.ini configuration file content.
    pub fn generate_pgbouncer_ini(
        &self,
        db_host: &str,
        db_port: u16,
        db_name: &str,
    ) -> Result<String, DomainError> {
        let config = self.config_snapshot.pgbouncer.as_ref().ok_or_else(|| {
            DomainError::Validation("PgBouncerConfig missing in config_snapshot".to_string())
        })?;

        let mut ini = String::new();
        ini.push_str("; Nodlance Managed PgBouncer Configuration\n");
        ini.push_str("[databases]\n");
        ini.push_str(&format!(
            "{} = host={} port={} dbname={}\n\n",
            db_name, db_host, db_port, db_name
        ));
        ini.push_str("[pgbouncer]\n");
        ini.push_str(&format!("listen_port = {}\n", self.listen_port));
        ini.push_str("listen_addr = *\n");
        ini.push_str(&format!("auth_type = {}\n", config.auth_type));
        ini.push_str("auth_file = /etc/pgbouncer/userlist.txt\n");
        ini.push_str("logfile = /var/log/pgbouncer/pgbouncer.log\n");
        ini.push_str("pidfile = /var/run/pgbouncer/pgbouncer.pid\n");
        ini.push_str(&format!("pool_mode = {}\n", config.pool_mode.as_str()));
        ini.push_str(&format!("max_client_conn = {}\n", config.max_client_conn));
        ini.push_str(&format!(
            "default_pool_size = {}\n",
            config.default_pool_size
        ));
        ini.push_str(&format!("min_pool_size = {}\n", config.min_pool_size));
        ini.push_str(&format!(
            "reserve_pool_size = {}\n",
            config.reserve_pool_size
        ));
        ini.push_str("reserve_pool_timeout = 5\n");
        ini.push_str("server_idle_timeout = 600\n");
        if let Some(admin) = self.admin_port {
            ini.push_str(&format!(
                "admin_users = nodlance_admin\n; admin port: {}\n",
                admin
            ));
        }

        Ok(ini)
    }

    /// Generates Keepalived VRRP virtual IP configuration.
    pub fn generate_keepalived_conf(&self) -> Result<String, DomainError> {
        let config = self.config_snapshot.keepalived.as_ref().ok_or_else(|| {
            DomainError::Validation("KeepalivedConfig missing in config_snapshot".to_string())
        })?;

        let mut conf = String::new();
        conf.push_str("! Nodlance Managed Keepalived Configuration\n");
        conf.push_str("global_defs {\n");
        conf.push_str("    router_id nodlance_lb\n");
        conf.push_str("    enable_script_security\n");
        conf.push_str("}\n\n");

        if let Some(script) = &config.track_script {
            conf.push_str("vrrp_script chk_db {\n");
            conf.push_str(&format!("    script \"{}\"\n", script));
            conf.push_str("    interval 2\n");
            conf.push_str("    weight 2\n");
            conf.push_str("}\n\n");
        }

        conf.push_str(&format!(
            "vrrp_instance VI_{} {{\n",
            config.virtual_router_id
        ));
        conf.push_str(&format!("    state {}\n", config.state));
        conf.push_str(&format!("    interface {}\n", config.interface));
        conf.push_str(&format!(
            "    virtual_router_id {}\n",
            config.virtual_router_id
        ));
        conf.push_str(&format!("    priority {}\n", config.priority));
        conf.push_str(&format!("    advert_int {}\n", config.advert_int_seconds));
        conf.push_str("    virtual_ipaddress {\n");
        conf.push_str(&format!("        {}\n", config.virtual_ip));
        conf.push_str("    }\n");
        if config.track_script.is_some() {
            conf.push_str("    track_script {\n");
            conf.push_str("        chk_db\n");
            conf.push_str("    }\n");
        }
        conf.push_str("}\n");

        Ok(conf)
    }
}

/// DTO for registering a new load balancer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewLoadBalancer {
    pub cluster_id: Uuid,
    pub host_id: Uuid,
    pub lb_type: LoadBalancerType,
    pub listen_port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_port: Option<u16>,
    #[serde(default)]
    pub config_snapshot: LoadBalancerConfigSnapshot,
}

impl NewLoadBalancer {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.listen_port == 0 {
            return Err(DomainError::Validation(
                "listen_port must be greater than 0".to_string(),
            ));
        }

        if let Some(admin) = self.admin_port {
            if admin == self.listen_port {
                return Err(DomainError::Validation(
                    "admin_port cannot be identical to listen_port".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// DTO for updating an existing load balancer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateLoadBalancer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<LoadBalancerStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_snapshot: Option<LoadBalancerConfigSnapshot>,
}

/// Query filter for load balancers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadBalancerFilter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lb_type: Option<LoadBalancerType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<LoadBalancerStatus>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_load_balancer_type_and_status_roundtrip() {
        assert_eq!(
            LoadBalancerType::from_str("proxysql").unwrap(),
            LoadBalancerType::ProxySql
        );
        assert_eq!(
            LoadBalancerType::from_str("pgbouncer").unwrap(),
            LoadBalancerType::PgBouncer
        );
        assert_eq!(
            LoadBalancerType::from_str("haproxy").unwrap(),
            LoadBalancerType::HaProxy
        );
        assert_eq!(
            LoadBalancerType::from_str("keepalived").unwrap(),
            LoadBalancerType::Keepalived
        );
        assert!(LoadBalancerType::from_str("unknown").is_err());

        assert_eq!(
            LoadBalancerStatus::from_str("active").unwrap(),
            LoadBalancerStatus::Active
        );
        assert_eq!(
            LoadBalancerStatus::from_str("standby").unwrap(),
            LoadBalancerStatus::Standby
        );
        assert_eq!(
            LoadBalancerStatus::from_str("degraded").unwrap(),
            LoadBalancerStatus::Degraded
        );
        assert_eq!(
            LoadBalancerStatus::from_str("error").unwrap(),
            LoadBalancerStatus::Error
        );
        assert_eq!(
            LoadBalancerStatus::from_str("maintenance").unwrap(),
            LoadBalancerStatus::Maintenance
        );
        assert!(LoadBalancerStatus::from_str("bad").is_err());
    }

    #[test]
    fn test_pgbouncer_pool_mode_roundtrip() {
        assert_eq!(
            PgBouncerPoolMode::from_str("session").unwrap(),
            PgBouncerPoolMode::Session
        );
        assert_eq!(
            PgBouncerPoolMode::from_str("transaction").unwrap(),
            PgBouncerPoolMode::Transaction
        );
        assert_eq!(
            PgBouncerPoolMode::from_str("statement").unwrap(),
            PgBouncerPoolMode::Statement
        );
        assert!(PgBouncerPoolMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_proxysql_admin_sql_generation() {
        let lb = LoadBalancer {
            id: Uuid::now_v7(),
            cluster_id: Uuid::now_v7(),
            host_id: Uuid::now_v7(),
            lb_type: LoadBalancerType::ProxySql,
            listen_port: 6033,
            admin_port: Some(6032),
            status: LoadBalancerStatus::Active,
            config_snapshot: LoadBalancerConfigSnapshot {
                proxysql: Some(ProxySqlConfig {
                    admin_username: "admin".to_string(),
                    writer_hostgroup: 10,
                    reader_hostgroup: 20,
                    servers: vec![
                        ProxySqlBackendServer {
                            hostgroup_id: 10,
                            hostname: "10.0.0.1".to_string(),
                            port: 3306,
                            weight: 100,
                            max_connections: 1000,
                            comment: Some("primary".to_string()),
                        },
                        ProxySqlBackendServer {
                            hostgroup_id: 20,
                            hostname: "10.0.0.2".to_string(),
                            port: 3306,
                            weight: 100,
                            max_connections: 1000,
                            comment: Some("replica".to_string()),
                        },
                    ],
                    query_rules: vec![
                        ProxySqlQueryRule {
                            rule_id: 1,
                            match_pattern: "^SELECT.*FOR UPDATE".to_string(),
                            destination_hostgroup: 10,
                            active: true,
                            comment: Some("write rule".to_string()),
                        },
                        ProxySqlQueryRule {
                            rule_id: 2,
                            match_pattern: "^SELECT".to_string(),
                            destination_hostgroup: 20,
                            active: true,
                            comment: Some("read rule".to_string()),
                        },
                    ],
                }),
                ..Default::default()
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        lb.validate().unwrap();
        let sql = lb.generate_proxysql_admin_sql().unwrap();
        assert!(sql.contains("INSERT INTO mysql_servers (hostgroup_id, hostname, port, weight, max_connections, comment) VALUES (10, '10.0.0.1', 3306, 100, 1000, 'primary');"));
        assert!(sql.contains("INSERT INTO mysql_servers (hostgroup_id, hostname, port, weight, max_connections, comment) VALUES (20, '10.0.0.2', 3306, 100, 1000, 'replica');"));
        assert!(sql.contains("LOAD MYSQL SERVERS TO RUNTIME;"));
        assert!(sql.contains("SAVE MYSQL QUERY RULES TO DISK;"));
    }

    #[test]
    fn test_pgbouncer_ini_generation() {
        let lb = LoadBalancer {
            id: Uuid::now_v7(),
            cluster_id: Uuid::now_v7(),
            host_id: Uuid::now_v7(),
            lb_type: LoadBalancerType::PgBouncer,
            listen_port: 6432,
            admin_port: Some(6433),
            status: LoadBalancerStatus::Active,
            config_snapshot: LoadBalancerConfigSnapshot {
                pgbouncer: Some(PgBouncerConfig {
                    pool_mode: PgBouncerPoolMode::Transaction,
                    max_client_conn: 500,
                    default_pool_size: 25,
                    min_pool_size: 5,
                    reserve_pool_size: 5,
                    auth_type: "scram-sha-256".to_string(),
                    users: vec!["appuser".to_string()],
                }),
                ..Default::default()
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        lb.validate().unwrap();
        let ini = lb
            .generate_pgbouncer_ini("10.0.0.5", 5432, "app_production")
            .unwrap();
        assert!(ini.contains("app_production = host=10.0.0.5 port=5432 dbname=app_production"));
        assert!(ini.contains("pool_mode = transaction"));
        assert!(ini.contains("max_client_conn = 500"));
        assert!(ini.contains("auth_type = scram-sha-256"));
    }

    #[test]
    fn test_keepalived_conf_generation() {
        let lb = LoadBalancer {
            id: Uuid::now_v7(),
            cluster_id: Uuid::now_v7(),
            host_id: Uuid::now_v7(),
            lb_type: LoadBalancerType::Keepalived,
            listen_port: 1, // virtual routing
            admin_port: None,
            status: LoadBalancerStatus::Active,
            config_snapshot: LoadBalancerConfigSnapshot {
                keepalived: Some(KeepalivedConfig {
                    virtual_router_id: 51,
                    interface: "eth0".to_string(),
                    virtual_ip: "10.0.0.100/24".to_string(),
                    priority: 101,
                    state: "MASTER".to_string(),
                    track_script: Some("/usr/local/bin/check_pg.sh".to_string()),
                    advert_int_seconds: 1,
                }),
                ..Default::default()
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let conf = lb.generate_keepalived_conf().unwrap();
        assert!(conf.contains("vrrp_instance VI_51 {"));
        assert!(conf.contains("state MASTER"));
        assert!(conf.contains("interface eth0"));
        assert!(conf.contains("10.0.0.100/24"));
        assert!(conf.contains("chk_db"));
    }

    #[test]
    fn test_invalid_ports_rejected() {
        let invalid_listen = NewLoadBalancer {
            cluster_id: Uuid::now_v7(),
            host_id: Uuid::now_v7(),
            lb_type: LoadBalancerType::ProxySql,
            listen_port: 0,
            admin_port: Some(6032),
            config_snapshot: Default::default(),
        };
        assert!(invalid_listen.validate().is_err());

        let identical_ports = NewLoadBalancer {
            cluster_id: Uuid::now_v7(),
            host_id: Uuid::now_v7(),
            lb_type: LoadBalancerType::ProxySql,
            listen_port: 6032,
            admin_port: Some(6032),
            config_snapshot: Default::default(),
        };
        assert!(identical_ports.validate().is_err());
    }
}
