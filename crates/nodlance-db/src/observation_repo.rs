//! PostgreSQL implementation of ObservationRepository.

use chrono::{DateTime, Utc};
use nodlance_domain::{
    ClusterRepository, ClusterTopology, DomainError, NodeHealth, NodeObservation, NodeRole,
    ObservationRepository,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{cluster_repo::PgClusterRepository, error::map_sqlx_error};

#[derive(Clone)]
pub struct PgObservationRepository {
    pool: PgPool,
    cluster_repo: PgClusterRepository,
}

impl PgObservationRepository {
    pub fn new(pool: PgPool) -> Self {
        let cluster_repo = PgClusterRepository::new(pool.clone());
        Self { pool, cluster_repo }
    }
}

#[derive(FromRow)]
struct ObservationRow {
    id: Uuid,
    node_id: Uuid,
    observed_at: DateTime<Utc>,
    role: String,
    health: String,
    replication_lag_seconds: Option<f64>,
    is_read_only: bool,
    details: Option<serde_json::Value>,
}

impl TryFrom<ObservationRow> for NodeObservation {
    type Error = DomainError;

    fn try_from(row: ObservationRow) -> Result<Self, Self::Error> {
        let role = match row.role.as_str() {
            "primary" => NodeRole::Primary,
            "replica" => NodeRole::Replica,
            "standalone" => NodeRole::Standalone,
            _ => NodeRole::Unknown,
        };

        let health = match row.health.as_str() {
            "healthy" => NodeHealth::Healthy,
            "degraded" => NodeHealth::Degraded,
            "unreachable" => NodeHealth::Unreachable,
            _ => NodeHealth::Unknown,
        };

        Ok(Self {
            id: row.id,
            node_id: row.node_id,
            observed_at: row.observed_at,
            role,
            health,
            replication_lag_seconds: row.replication_lag_seconds,
            is_read_only: row.is_read_only,
            details: row.details,
        })
    }
}

impl ObservationRepository for PgObservationRepository {
    async fn record_observation(
        &self,
        node_id: Uuid,
        role: NodeRole,
        health: NodeHealth,
        lag: Option<f64>,
        read_only: bool,
        details: Option<serde_json::Value>,
    ) -> Result<NodeObservation, DomainError> {
        let id = Uuid::now_v7();

        let role_str = match role {
            NodeRole::Primary => "primary",
            NodeRole::Replica => "replica",
            NodeRole::Standalone => "standalone",
            NodeRole::Unknown => "unknown",
        };

        let health_str = match health {
            NodeHealth::Healthy => "healthy",
            NodeHealth::Degraded => "degraded",
            NodeHealth::Unreachable => "unreachable",
            NodeHealth::Unknown => "unknown",
        };

        let row = sqlx::query_as::<_, ObservationRow>(
            r#"
            INSERT INTO node_observations (id, node_id, observed_at, role, health, replication_lag_seconds, is_read_only, details)
            VALUES ($1, $2, now(), $3, $4, $5, $6, $7)
            RETURNING id, node_id, observed_at, role, health, replication_lag_seconds, is_read_only, details
            "#,
        )
        .bind(id)
        .bind(node_id)
        .bind(role_str)
        .bind(health_str)
        .bind(lag)
        .bind(read_only)
        .bind(details)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "record node observation"))?;

        // Update current status on database_nodes table
        let _ = self
            .cluster_repo
            .update_node_status(node_id, role, health)
            .await;

        row.try_into()
    }

    async fn get_latest_observation(
        &self,
        node_id: Uuid,
    ) -> Result<Option<NodeObservation>, DomainError> {
        let row = sqlx::query_as::<_, ObservationRow>(
            r#"
            SELECT id, node_id, observed_at, role, health, replication_lag_seconds, is_read_only, details
            FROM node_observations
            WHERE node_id = $1
            ORDER BY observed_at DESC
            LIMIT 1
            "#,
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get latest observation"))?;

        row.map(NodeObservation::try_from).transpose()
    }

    async fn get_cluster_topology(&self, cluster_id: Uuid) -> Result<ClusterTopology, DomainError> {
        let nodes = self.cluster_repo.list_nodes_by_cluster(cluster_id).await?;

        let mut observations = Vec::new();
        let mut primary_node_id = None;

        for node in &nodes {
            if let Some(obs) = self.get_latest_observation(node.id).await? {
                if obs.role == NodeRole::Primary && obs.health == NodeHealth::Healthy {
                    primary_node_id = Some(node.id);
                }
                observations.push(obs);
            }
        }

        Ok(ClusterTopology {
            cluster_id,
            nodes,
            observations,
            primary_node_id,
        })
    }
}
