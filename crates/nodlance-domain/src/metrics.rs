//! Host metrics telemetry data models.

use serde::{Deserialize, Serialize};

/// Host metrics snapshot captured by the node agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMetrics {
    pub cpu_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub load_avg_one: f32,
}
