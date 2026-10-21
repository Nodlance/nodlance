//! Host inventory types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Registration status of a managed host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostStatus {
    Active,
    Inactive,
    Decommissioned,
}

impl HostStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Decommissioned => "decommissioned",
        }
    }
}

impl std::str::FromStr for HostStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "decommissioned" => Ok(Self::Decommissioned),
            other => Err(format!("unknown host status: {other}")),
        }
    }
}

/// A managed host in the inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub id: Uuid,
    pub hostname: String,
    pub ip_address: Option<String>,
    pub os_family: Option<String>,
    pub os_version: Option<String>,
    pub labels: serde_json::Value,
    pub status: HostStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

/// Input for registering a new host.
#[derive(Debug, Deserialize)]
pub struct NewHost {
    pub hostname: String,
    pub ip_address: Option<String>,
    pub os_family: Option<String>,
    pub os_version: Option<String>,
    pub labels: Option<serde_json::Value>,
}

/// Partial update to an existing host.
#[derive(Debug, Deserialize)]
pub struct UpdateHost {
    pub hostname: Option<String>,
    pub ip_address: Option<String>,
    pub os_family: Option<String>,
    pub os_version: Option<String>,
    pub labels: Option<serde_json::Value>,
    pub status: Option<HostStatus>,
}

/// Filtering parameters for host listing queries.
#[derive(Debug, Default, Deserialize)]
pub struct HostFilter {
    pub status: Option<HostStatus>,
    pub hostname_contains: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::HostStatus;

    #[test]
    fn host_status_round_trips() {
        for status in [
            HostStatus::Active,
            HostStatus::Inactive,
            HostStatus::Decommissioned,
        ] {
            let parsed: HostStatus = status.as_str().parse().expect("should parse");
            assert_eq!(parsed, status);
        }
    }
}
