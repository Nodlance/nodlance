//! Repository ports (traits) for domain entity persistence.

use std::future::Future;
use uuid::Uuid;

use crate::{
    agent::AgentIdentity,
    alert::{AlertIncident, AlertRule, MaintenanceWindow, NewMaintenanceWindow},
    audit::{AuditEvent, AuditFilter, NewAuditEvent},
    cluster::{Cluster, DatabaseNode, NewCluster, NewDatabaseNode},
    engine::{NodeHealth, NodeRole},
    error::DomainError,
    heartbeat::AgentHeartbeat,
    host::{Host, HostFilter, NewHost, UpdateHost},
    metrics::HostMetrics,
    observation::{ClusterTopology, NodeObservation},
    user::{NewUser, Permission, Role, User},
};

/// Port for host inventory persistence operations.
pub trait HostRepository: Send + Sync {
    fn create(
        &self,
        created_by: Uuid,
        new_host: &NewHost,
    ) -> impl Future<Output = Result<Host, DomainError>> + Send;

    fn get_by_id(&self, id: Uuid)
    -> impl Future<Output = Result<Option<Host>, DomainError>> + Send;

    fn list(
        &self,
        filter: &HostFilter,
    ) -> impl Future<Output = Result<Vec<Host>, DomainError>> + Send;

    fn update(
        &self,
        id: Uuid,
        update: &UpdateHost,
    ) -> impl Future<Output = Result<Host, DomainError>> + Send;

    fn decommission(&self, id: Uuid) -> impl Future<Output = Result<Host, DomainError>> + Send;
}

/// Port for agent enrollment and lifecycle persistence.
pub trait AgentRepository: Send + Sync {
    fn enroll(
        &self,
        host_id: Uuid,
        fingerprint: &str,
    ) -> impl Future<Output = Result<AgentIdentity, DomainError>> + Send;

    fn get_by_host_id(
        &self,
        host_id: Uuid,
    ) -> impl Future<Output = Result<Option<AgentIdentity>, DomainError>> + Send;

    fn get_by_fingerprint(
        &self,
        fingerprint: &str,
    ) -> impl Future<Output = Result<Option<AgentIdentity>, DomainError>> + Send;

    fn revoke(
        &self,
        host_id: Uuid,
        revoked_by: Uuid,
    ) -> impl Future<Output = Result<AgentIdentity, DomainError>> + Send;
}

/// Port for user identity, role, and permission persistence.
pub trait UserRepository: Send + Sync {
    fn create(
        &self,
        new_user: &NewUser,
        password_hash: &str,
    ) -> impl Future<Output = Result<User, DomainError>> + Send;

    fn get_by_id(&self, id: Uuid)
    -> impl Future<Output = Result<Option<User>, DomainError>> + Send;

    fn get_by_username(
        &self,
        username: &str,
    ) -> impl Future<Output = Result<Option<User>, DomainError>> + Send;

    fn list(&self) -> impl Future<Output = Result<Vec<User>, DomainError>> + Send;

    fn get_user_roles(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, DomainError>> + Send;

    fn get_user_permissions(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Permission>, DomainError>> + Send;

    fn assign_role(
        &self,
        user_id: Uuid,
        role_name: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
}

/// Port for append-only audit event logging and querying.
pub trait AuditRepository: Send + Sync {
    fn record(
        &self,
        event: &NewAuditEvent,
    ) -> impl Future<Output = Result<AuditEvent, DomainError>> + Send;

    fn list(
        &self,
        filter: &AuditFilter,
    ) -> impl Future<Output = Result<Vec<AuditEvent>, DomainError>> + Send;
}

/// Port for database cluster and node persistence.
pub trait ClusterRepository: Send + Sync {
    fn create_cluster(
        &self,
        new_cluster: &NewCluster,
    ) -> impl Future<Output = Result<Cluster, DomainError>> + Send;

    fn get_cluster_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Cluster>, DomainError>> + Send;

    fn list_clusters(&self) -> impl Future<Output = Result<Vec<Cluster>, DomainError>> + Send;

    fn create_node(
        &self,
        new_node: &NewDatabaseNode,
    ) -> impl Future<Output = Result<DatabaseNode, DomainError>> + Send;

    fn get_node_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<DatabaseNode>, DomainError>> + Send;

    fn list_nodes_by_cluster(
        &self,
        cluster_id: Uuid,
    ) -> impl Future<Output = Result<Vec<DatabaseNode>, DomainError>> + Send;

    fn update_node_status(
        &self,
        node_id: Uuid,
        role: NodeRole,
        health: NodeHealth,
    ) -> impl Future<Output = Result<DatabaseNode, DomainError>> + Send;
}

/// Port for agent heartbeats and host metrics persistence.
pub trait HeartbeatRepository: Send + Sync {
    fn record_heartbeat(
        &self,
        host_id: Uuid,
        agent_version: &str,
        metrics: Option<&HostMetrics>,
    ) -> impl Future<Output = Result<AgentHeartbeat, DomainError>> + Send;

    fn get_latest_heartbeat(
        &self,
        host_id: Uuid,
    ) -> impl Future<Output = Result<Option<AgentHeartbeat>, DomainError>> + Send;

    fn get_metrics_history(
        &self,
        host_id: Uuid,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<AgentHeartbeat>, DomainError>> + Send;
}

/// Port for engine health and replication topology observations.
pub trait ObservationRepository: Send + Sync {
    fn record_observation(
        &self,
        node_id: Uuid,
        role: NodeRole,
        health: NodeHealth,
        lag: Option<f64>,
        read_only: bool,
        details: Option<serde_json::Value>,
    ) -> impl Future<Output = Result<NodeObservation, DomainError>> + Send;

    fn get_latest_observation(
        &self,
        node_id: Uuid,
    ) -> impl Future<Output = Result<Option<NodeObservation>, DomainError>> + Send;

    fn get_cluster_topology(
        &self,
        cluster_id: Uuid,
    ) -> impl Future<Output = Result<ClusterTopology, DomainError>> + Send;
}

/// Port for alerts and maintenance windows persistence.
pub trait AlertRepository: Send + Sync {
    fn list_rules(&self) -> impl Future<Output = Result<Vec<AlertRule>, DomainError>> + Send;

    fn record_incident(
        &self,
        incident: &AlertIncident,
    ) -> impl Future<Output = Result<AlertIncident, DomainError>> + Send;

    fn list_active_incidents(
        &self,
    ) -> impl Future<Output = Result<Vec<AlertIncident>, DomainError>> + Send;

    fn acknowledge_incident(
        &self,
        incident_id: Uuid,
    ) -> impl Future<Output = Result<AlertIncident, DomainError>> + Send;

    fn create_maintenance_window(
        &self,
        window: &NewMaintenanceWindow,
    ) -> impl Future<Output = Result<MaintenanceWindow, DomainError>> + Send;

    fn is_in_maintenance(
        &self,
        resource_type: &str,
        resource_id: Option<Uuid>,
    ) -> impl Future<Output = Result<bool, DomainError>> + Send;
}
