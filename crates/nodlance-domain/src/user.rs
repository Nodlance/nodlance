//! User identity, roles, and permissions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A control-plane user account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    /// Never serialized to API responses.
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new user account.
#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub display_name: Option<String>,
}

/// A named collection of permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

/// Individual permissions for RBAC enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    #[serde(rename = "user:read")]
    UserRead,
    #[serde(rename = "user:write")]
    UserWrite,
    #[serde(rename = "host:read")]
    HostRead,
    #[serde(rename = "host:write")]
    HostWrite,
    #[serde(rename = "agent:write")]
    AgentWrite,
    #[serde(rename = "audit:read")]
    AuditRead,
}

impl Permission {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserRead => "user:read",
            Self::UserWrite => "user:write",
            Self::HostRead => "host:read",
            Self::HostWrite => "host:write",
            Self::AgentWrite => "agent:write",
            Self::AuditRead => "audit:read",
        }
    }
}

impl std::str::FromStr for Permission {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user:read" => Ok(Self::UserRead),
            "user:write" => Ok(Self::UserWrite),
            "host:read" => Ok(Self::HostRead),
            "host:write" => Ok(Self::HostWrite),
            "agent:write" => Ok(Self::AgentWrite),
            "audit:read" => Ok(Self::AuditRead),
            other => Err(format!("unknown permission: {other}")),
        }
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::Permission;

    #[test]
    fn permission_round_trips() {
        for perm in [
            Permission::UserRead,
            Permission::UserWrite,
            Permission::HostRead,
            Permission::HostWrite,
            Permission::AgentWrite,
            Permission::AuditRead,
        ] {
            let parsed: Permission = perm.as_str().parse().expect("should parse");
            assert_eq!(parsed, perm);
        }
    }

    #[test]
    fn permission_serializes_to_colon_notation() {
        let json = serde_json::to_string(&Permission::HostWrite)
            .unwrap_or_else(|e| panic!("serialization failed: {e}"));
        assert_eq!(json, "\"host:write\"");
    }
}
