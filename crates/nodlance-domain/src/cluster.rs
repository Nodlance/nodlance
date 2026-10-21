//! Database clusters and member nodes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::{DatabaseEngine, NodeHealth, NodeRole};

/// A logical database cluster grouping one or more database nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub id: Uuid,
    pub name: String,
    pub engine: DatabaseEngine,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new database cluster.
#[derive(Debug, Deserialize)]
pub struct NewCluster {
    pub name: String,
    pub engine: DatabaseEngine,
}

/// A database instance running on a managed host belonging to a cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseNode {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub host_id: Uuid,
    pub name: String,
    pub port: u16,
    pub role: NodeRole,
    pub health: NodeHealth,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for registering a database node into a cluster.
#[derive(Debug, Deserialize)]
pub struct NewDatabaseNode {
    pub cluster_id: Uuid,
    pub host_id: Uuid,
    pub name: String,
    pub port: u16,
}
