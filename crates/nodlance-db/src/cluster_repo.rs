//! PostgreSQL implementation of ClusterRepository.

use chrono::{DateTime, Utc};
use nodlance_domain::{
    Cluster, ClusterRepository, DatabaseEngine, DatabaseNode, DomainError, NewCluster,
    NewDatabaseNode, NodeHealth, NodeRole,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgClusterRepository {
    pool: PgPool,
}

impl PgClusterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct ClusterRow {
    id: Uuid,
    name: String,
    engine: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ClusterRow> for Cluster {
    type Error = DomainError;

    fn try_from(row: ClusterRow) -> Result<Self, Self::Error> {
        let engine = match row.engine.as_str() {
            "postgresql" => DatabaseEngine::PostgreSql,
            "mysql" => DatabaseEngine::MySql,
            "mariadb" => DatabaseEngine::MariaDb,
            other => {
                return Err(DomainError::Internal(format!(
                    "unknown database engine in db: {other}"
                )));
            }
        };

        Ok(Self {
            id: row.id,
            name: row.name,
            engine,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(FromRow)]
struct NodeRow {
    id: Uuid,
    cluster_id: Uuid,
    host_id: Uuid,
    name: String,
    port: i32,
    role: String,
    health: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<NodeRow> for DatabaseNode {
    type Error = DomainError;

    fn try_from(row: NodeRow) -> Result<Self, Self::Error> {
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
            cluster_id: row.cluster_id,
            host_id: row.host_id,
            name: row.name,
            port: row.port as u16,
            role,
            health,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl ClusterRepository for PgClusterRepository {
    async fn create_cluster(&self, new_cluster: &NewCluster) -> Result<Cluster, DomainError> {
        let id = Uuid::now_v7();

        let row = sqlx::query_as::<_, ClusterRow>(
            r#"
            INSERT INTO clusters (id, name, engine, created_at, updated_at)
            VALUES ($1, $2, $3, now(), now())
            RETURNING id, name, engine, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&new_cluster.name)
        .bind(new_cluster.engine.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "create cluster"))?;

        row.try_into()
    }

    async fn get_cluster_by_id(&self, id: Uuid) -> Result<Option<Cluster>, DomainError> {
        let row = sqlx::query_as::<_, ClusterRow>(
            r#"
            SELECT id, name, engine, created_at, updated_at
            FROM clusters
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get cluster by id"))?;

        row.map(Cluster::try_from).transpose()
    }

    async fn list_clusters(&self) -> Result<Vec<Cluster>, DomainError> {
        let rows = sqlx::query_as::<_, ClusterRow>(
            r#"
            SELECT id, name, engine, created_at, updated_at
            FROM clusters
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list clusters"))?;

        rows.into_iter().map(Cluster::try_from).collect()
    }

    async fn create_node(&self, new_node: &NewDatabaseNode) -> Result<DatabaseNode, DomainError> {
        let id = Uuid::now_v7();

        let row = sqlx::query_as::<_, NodeRow>(
            r#"
            INSERT INTO database_nodes (id, cluster_id, host_id, name, port, role, health, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, 'unknown', 'unknown', now(), now())
            RETURNING id, cluster_id, host_id, name, port, role, health, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(new_node.cluster_id)
        .bind(new_node.host_id)
        .bind(&new_node.name)
        .bind(new_node.port as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "create database node"))?;

        row.try_into()
    }

    async fn get_node_by_id(&self, id: Uuid) -> Result<Option<DatabaseNode>, DomainError> {
        let row = sqlx::query_as::<_, NodeRow>(
            r#"
            SELECT id, cluster_id, host_id, name, port, role, health, created_at, updated_at
            FROM database_nodes
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get database node by id"))?;

        row.map(DatabaseNode::try_from).transpose()
    }

    async fn list_nodes_by_cluster(
        &self,
        cluster_id: Uuid,
    ) -> Result<Vec<DatabaseNode>, DomainError> {
        let rows = sqlx::query_as::<_, NodeRow>(
            r#"
            SELECT id, cluster_id, host_id, name, port, role, health, created_at, updated_at
            FROM database_nodes
            WHERE cluster_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(cluster_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list database nodes by cluster"))?;

        rows.into_iter().map(DatabaseNode::try_from).collect()
    }

    async fn update_node_status(
        &self,
        node_id: Uuid,
        role: NodeRole,
        health: NodeHealth,
    ) -> Result<DatabaseNode, DomainError> {
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

        let row = sqlx::query_as::<_, NodeRow>(
            r#"
            UPDATE database_nodes
            SET role = $2, health = $3, updated_at = now()
            WHERE id = $1
            RETURNING id, cluster_id, host_id, name, port, role, health, created_at, updated_at
            "#,
        )
        .bind(node_id)
        .bind(role_str)
        .bind(health_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "update node status"))?
        .ok_or_else(|| DomainError::NotFound(format!("node not found: {node_id}")))?;

        row.try_into()
    }
}
