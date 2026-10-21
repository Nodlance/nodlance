//! Database engine vocabulary and state enumerations.

use serde::{Deserialize, Serialize};

/// Database engine families Nodlance plans to manage.
///
/// This enum describes the domain vocabulary only; it does not imply that a
/// given engine/version has been integrated or tested yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatabaseEngine {
    #[serde(rename = "postgresql")]
    PostgreSql,
    #[serde(rename = "mysql")]
    MySql,
    #[serde(rename = "mariadb")]
    MariaDb,
}

impl DatabaseEngine {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PostgreSql => "postgresql",
            Self::MySql => "mysql",
            Self::MariaDb => "mariadb",
        }
    }
}

/// The role observed for a database node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeRole {
    Primary,
    Replica,
    Standalone,
    Unknown,
}

/// Health is an observation, not proof that a node can or cannot accept writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeHealth {
    Healthy,
    Degraded,
    Unreachable,
    Unknown,
}

/// Lifecycle state for persisted jobs added in a later milestone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    UnknownOutcome,
    ReconciliationRequired,
}

#[cfg(test)]
mod tests {
    use super::DatabaseEngine;

    #[test]
    fn engine_wire_names_are_stable() {
        assert_eq!(DatabaseEngine::PostgreSql.as_str(), "postgresql");
        assert_eq!(DatabaseEngine::MySql.as_str(), "mysql");
        assert_eq!(DatabaseEngine::MariaDb.as_str(), "mariadb");
    }

    #[test]
    fn engine_serializes_to_expected_api_value() {
        let actual = serde_json::to_string(&DatabaseEngine::PostgreSql)
            .unwrap_or_else(|error| panic!("serialization failed: {error}"));
        assert_eq!(actual, "\"postgresql\"");
    }
}
