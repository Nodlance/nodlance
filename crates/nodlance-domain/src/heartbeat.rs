//! Agent heartbeat and telemetry ingestion.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::metrics::HostMetrics;

/// Recorded heartbeat observation from an enrolled agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHeartbeat {
    pub id: Uuid,
    pub host_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub agent_version: String,
    pub metrics: Option<HostMetrics>,
}

/// Inbound payload sent by the node agent periodically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub fingerprint: String,
    pub agent_version: String,
    pub metrics: Option<HostMetrics>,
}
