//! Agent identity and enrollment types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle status of a node agent identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Pending,
    Enrolled,
    Revoked,
}

impl AgentStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Enrolled => "enrolled",
            Self::Revoked => "revoked",
        }
    }
}

impl std::str::FromStr for AgentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "enrolled" => Ok(Self::Enrolled),
            "revoked" => Ok(Self::Revoked),
            other => Err(format!("unknown agent status: {other}")),
        }
    }
}

/// Enrolled agent identity for a managed host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub id: Uuid,
    pub host_id: Uuid,
    pub fingerprint: String,
    pub status: AgentStatus,
    pub enrolled_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Request body for agent enrollment.
#[derive(Debug, Deserialize)]
pub struct EnrollmentRequest {
    pub fingerprint: String,
}
