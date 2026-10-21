//! Audit event types for tracking privileged operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An immutable record of a privileged action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub source_ip: Option<String>,
}

/// Input for recording a new audit event.
#[derive(Debug)]
pub struct NewAuditEvent {
    pub actor_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub source_ip: Option<String>,
}

/// Filtering parameters for audit event queries.
#[derive(Debug, Default, Deserialize)]
pub struct AuditFilter {
    pub actor_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
