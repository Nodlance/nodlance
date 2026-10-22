//! Domain models for scriptable DBA heuristics, advisor rules & findings,
//! and normalized query performance digest telemetry.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

/// Severity classification for DBA advisor findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdvisorSeverity {
    Info,
    Warning,
    Critical,
}

impl AdvisorSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

impl fmt::Display for AdvisorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AdvisorSeverity {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(Self::Info),
            "warning" | "warn" => Ok(Self::Warning),
            "critical" | "crit" | "danger" => Ok(Self::Critical),
            other => Err(DomainError::Validation(format!(
                "invalid advisor severity: '{other}'"
            ))),
        }
    }
}

/// Operational category for DBA advisor rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdvisorCategory {
    Performance,
    Security,
    Capacity,
    Availability,
}

impl AdvisorCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Performance => "performance",
            Self::Security => "security",
            Self::Capacity => "capacity",
            Self::Availability => "availability",
        }
    }
}

impl fmt::Display for AdvisorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AdvisorCategory {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "performance" | "perf" => Ok(Self::Performance),
            "security" | "sec" => Ok(Self::Security),
            "capacity" | "storage" => Ok(Self::Capacity),
            "availability" | "ha" => Ok(Self::Availability),
            other => Err(DomainError::Validation(format!(
                "invalid advisor category: '{other}'"
            ))),
        }
    }
}

/// A scriptable DBA heuristic rule definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvisorRule {
    pub id: String,
    pub name: String,
    pub engine: String, // 'postgres', 'mysql', 'galera', 'any'
    pub category: AdvisorCategory,
    pub severity: AdvisorSeverity,
    pub description: String,
    pub script_content: String,
    pub is_enabled: bool,
}

impl AdvisorRule {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.id.trim().is_empty() {
            return Err(DomainError::Validation(
                "advisor rule id cannot be empty".to_string(),
            ));
        }
        if self.name.trim().is_empty() {
            return Err(DomainError::Validation(
                "advisor rule name cannot be empty".to_string(),
            ));
        }
        if self.script_content.trim().is_empty() {
            return Err(DomainError::Validation(
                "advisor rule script_content cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

/// A specific anomaly or optimization recommendation identified by an advisor rule scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvisorFinding {
    pub id: Uuid,
    pub rule_id: String,
    pub rule_name: String,
    pub cluster_id: Uuid,
    pub node_id: Option<Uuid>,
    pub severity: AdvisorSeverity,
    pub summary: String,
    pub recommendation: String,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Normalized SQL statement performance digest (from `pg_stat_statements` or `performance_schema`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryDigest {
    pub query_hash: String,
    pub cluster_id: Uuid,
    pub node_id: Option<Uuid>,
    pub database_name: String,
    pub digest_text: String,
    pub execution_count: u64,
    pub total_latency_ms: f64,
    pub mean_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub max_latency_ms: f64,
    pub rows_examined: u64,
    pub rows_sent: u64,
    pub last_seen: DateTime<Utc>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_advisor_severity_and_category_parse() {
        assert_eq!(
            AdvisorSeverity::from_str("warning").unwrap(),
            AdvisorSeverity::Warning
        );
        assert_eq!(
            AdvisorSeverity::from_str("critical").unwrap(),
            AdvisorSeverity::Critical
        );
        assert_eq!(
            AdvisorSeverity::from_str("info").unwrap(),
            AdvisorSeverity::Info
        );
        assert!(AdvisorSeverity::from_str("bogus").is_err());

        assert_eq!(
            AdvisorCategory::from_str("performance").unwrap(),
            AdvisorCategory::Performance
        );
        assert_eq!(
            AdvisorCategory::from_str("security").unwrap(),
            AdvisorCategory::Security
        );
        assert_eq!(
            AdvisorCategory::from_str("capacity").unwrap(),
            AdvisorCategory::Capacity
        );
        assert_eq!(
            AdvisorCategory::from_str("availability").unwrap(),
            AdvisorCategory::Availability
        );
        assert!(AdvisorCategory::from_str("invalid").is_err());
    }

    #[test]
    fn test_advisor_rule_validation() {
        let valid = AdvisorRule {
            id: "rule_buffer_pool".to_string(),
            name: "InnoDB Buffer Pool Utilization".to_string(),
            engine: "mysql".to_string(),
            category: AdvisorCategory::Performance,
            severity: AdvisorSeverity::Warning,
            description: "Alerts when buffer pool hit ratio is < 95%".to_string(),
            script_content: "if hit_ratio < 0.95 { trigger_finding(); }".to_string(),
            is_enabled: true,
        };
        assert!(valid.validate().is_ok());

        let empty_id = AdvisorRule {
            id: " ".to_string(),
            ..valid.clone()
        };
        assert!(empty_id.validate().is_err());

        let empty_script = AdvisorRule {
            script_content: "".to_string(),
            ..valid
        };
        assert!(empty_script.validate().is_err());
    }
}
