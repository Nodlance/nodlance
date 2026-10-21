//! Database engine health and replication observations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    cluster::DatabaseNode,
    engine::{NodeHealth, NodeRole},
};

/// An engine health and replication observation snapshot for a single database node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeObservation {
    pub id: Uuid,
    pub node_id: Uuid,
    pub observed_at: DateTime<Utc>,
    pub role: NodeRole,
    pub health: NodeHealth,
    pub replication_lag_seconds: Option<f64>,
    pub is_read_only: bool,
    pub details: Option<serde_json::Value>,
}

/// Observed topology view for an entire database cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterTopology {
    pub cluster_id: Uuid,
    pub nodes: Vec<DatabaseNode>,
    pub observations: Vec<NodeObservation>,
    pub primary_node_id: Option<Uuid>,
}
