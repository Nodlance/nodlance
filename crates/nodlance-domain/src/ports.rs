//! Repository ports (traits) for domain entity persistence.

use std::future::Future;
use uuid::Uuid;

use crate::{
    agent::AgentIdentity,
    alert::{AlertIncident, AlertRule, MaintenanceWindow, NewMaintenanceWindow},
    audit::{AuditEvent, AuditFilter, NewAuditEvent},
    backup::{BackupArtifact, NewBackupArtifact, RestoreValidationDrill},
    cluster::{Cluster, DatabaseNode, NewCluster, NewDatabaseNode},
    engine::{NodeHealth, NodeRole},
    error::DomainError,
    heartbeat::AgentHeartbeat,
    host::{Host, HostFilter, NewHost, UpdateHost},
    job::{Job, JobAttempt, NewJob},
    metrics::HostMetrics,
    observation::{ClusterTopology, NodeObservation},
    provisioning::{
        DiscoveredWorkload, NewDiscoveredWorkload, NewProvisioningPlan, ProviderType,
        ProvisioningPlan, ProvisioningStatus, ProvisioningStep, StepStatus,
    },
    recovery::{
        ClusterEpoch, FencingExecution, FencingStatus, FencingType, NewPromotionRequest,
        PromotionRequest, UpdatePromotionParams,
    },
    load_balancer::{LoadBalancer, LoadBalancerFilter, NewLoadBalancer, UpdateLoadBalancer},
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

/// Port for durable background job execution, leasing, and retries.
pub trait JobRepository: Send + Sync {
    fn enqueue_job(
        &self,
        new_job: &NewJob,
    ) -> impl Future<Output = Result<Job, DomainError>> + Send;

    fn acquire_lease(
        &self,
        lease_owner: &str,
        lease_duration_secs: u64,
    ) -> impl Future<Output = Result<Option<Job>, DomainError>> + Send;

    fn record_attempt_start(
        &self,
        job_id: Uuid,
        attempt_number: i32,
    ) -> impl Future<Output = Result<JobAttempt, DomainError>> + Send;

    fn complete_job(
        &self,
        job_id: Uuid,
        attempt_id: Uuid,
        result: serde_json::Value,
    ) -> impl Future<Output = Result<Job, DomainError>> + Send;

    fn fail_job(
        &self,
        job_id: Uuid,
        attempt_id: Uuid,
        error_message: &str,
        can_retry: bool,
    ) -> impl Future<Output = Result<Job, DomainError>> + Send;

    fn get_job_by_id(
        &self,
        job_id: Uuid,
    ) -> impl Future<Output = Result<Option<Job>, DomainError>> + Send;

    fn list_jobs(&self, limit: i64) -> impl Future<Output = Result<Vec<Job>, DomainError>> + Send;
}

/// Port for backup artifacts, checksums, retention, and verification drills.
pub trait BackupRepository: Send + Sync {
    fn record_backup(
        &self,
        artifact: &NewBackupArtifact,
    ) -> impl Future<Output = Result<BackupArtifact, DomainError>> + Send;

    fn get_backup_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<BackupArtifact>, DomainError>> + Send;

    fn list_backups_by_cluster(
        &self,
        cluster_id: Uuid,
    ) -> impl Future<Output = Result<Vec<BackupArtifact>, DomainError>> + Send;

    fn record_restore_drill(
        &self,
        backup_id: Uuid,
        is_valid: bool,
        report: serde_json::Value,
    ) -> impl Future<Output = Result<RestoreValidationDrill, DomainError>> + Send;

    fn get_latest_drill(
        &self,
        backup_id: Uuid,
    ) -> impl Future<Output = Result<Option<RestoreValidationDrill>, DomainError>> + Send;

    fn cleanup_expired_backups(&self) -> impl Future<Output = Result<u64, DomainError>> + Send;
}

/// Port for guarded provisioning plans, step tracking, and rollback logs.
pub trait ProvisioningRepository: Send + Sync {
    fn create_plan(
        &self,
        plan: &NewProvisioningPlan,
    ) -> impl Future<Output = Result<ProvisioningPlan, DomainError>> + Send;

    fn get_plan_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<ProvisioningPlan>, DomainError>> + Send;

    fn list_plans(
        &self,
        cluster_id: Option<Uuid>,
    ) -> impl Future<Output = Result<Vec<ProvisioningPlan>, DomainError>> + Send;

    fn update_plan_status(
        &self,
        id: Uuid,
        status: ProvisioningStatus,
    ) -> impl Future<Output = Result<ProvisioningPlan, DomainError>> + Send;

    fn record_step(
        &self,
        step: &ProvisioningStep,
    ) -> impl Future<Output = Result<ProvisioningStep, DomainError>> + Send;

    fn list_steps_for_plan(
        &self,
        plan_id: Uuid,
    ) -> impl Future<Output = Result<Vec<ProvisioningStep>, DomainError>> + Send;

    fn update_step_status(
        &self,
        step_id: Uuid,
        status: StepStatus,
        error_details: Option<&str>,
    ) -> impl Future<Output = Result<ProvisioningStep, DomainError>> + Send;
}

/// Port for recording and querying discovered database workloads.
pub trait DiscoveryRepository: Send + Sync {
    fn record_discovered_workload(
        &self,
        workload: &NewDiscoveredWorkload,
    ) -> impl Future<Output = Result<DiscoveredWorkload, DomainError>> + Send;

    fn list_discovered_workloads(
        &self,
        provider_type: Option<ProviderType>,
    ) -> impl Future<Output = Result<Vec<DiscoveredWorkload>, DomainError>> + Send;

    fn get_discovered_workload_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<DiscoveredWorkload>, DomainError>> + Send;
}

/// Port for cluster recovery, epoch monotonic tracking, promotion workflows, and fencing.
pub trait RecoveryRepository: Send + Sync {
    fn get_current_epoch(
        &self,
        cluster_id: Uuid,
    ) -> impl Future<Output = Result<Option<ClusterEpoch>, DomainError>> + Send;

    fn increment_epoch(
        &self,
        cluster_id: Uuid,
        promoted_host_id: Option<Uuid>,
        reason: &str,
    ) -> impl Future<Output = Result<ClusterEpoch, DomainError>> + Send;

    fn create_promotion_request(
        &self,
        req: &NewPromotionRequest,
    ) -> impl Future<Output = Result<PromotionRequest, DomainError>> + Send;

    fn get_promotion_request(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<PromotionRequest>, DomainError>> + Send;

    fn list_promotion_requests(
        &self,
        cluster_id: Option<Uuid>,
    ) -> impl Future<Output = Result<Vec<PromotionRequest>, DomainError>> + Send;

    fn update_promotion_status(
        &self,
        id: Uuid,
        params: UpdatePromotionParams<'_>,
    ) -> impl Future<Output = Result<PromotionRequest, DomainError>> + Send;

    fn record_fencing_execution(
        &self,
        execution: &FencingExecution,
    ) -> impl Future<Output = Result<FencingExecution, DomainError>> + Send;

    fn get_fencing_execution(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<FencingExecution>, DomainError>> + Send;

    fn update_fencing_status(
        &self,
        id: Uuid,
        status: FencingStatus,
        verification_proof: Option<&str>,
        error_message: Option<&str>,
    ) -> impl Future<Output = Result<FencingExecution, DomainError>> + Send;
}

/// Port for dispatching node fencing operations.
pub trait FencingProvider: Send + Sync {
    fn execute_fencing(
        &self,
        cluster_id: Uuid,
        target_host_id: Uuid,
        fencing_type: FencingType,
    ) -> impl Future<Output = Result<FencingExecution, DomainError>> + Send;

    fn verify_fencing(
        &self,
        execution_id: Uuid,
    ) -> impl Future<Output = Result<bool, DomainError>> + Send;
}

/// Port for load balancer and connection router persistence operations.
pub trait LoadBalancerRepository: Send + Sync {
    fn create(
        &self,
        new_lb: &NewLoadBalancer,
    ) -> impl Future<Output = Result<LoadBalancer, DomainError>> + Send;

    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<LoadBalancer>, DomainError>> + Send;

    fn list(
        &self,
        filter: &LoadBalancerFilter,
    ) -> impl Future<Output = Result<Vec<LoadBalancer>, DomainError>> + Send;

    fn update(
        &self,
        id: Uuid,
        update: &UpdateLoadBalancer,
    ) -> impl Future<Output = Result<LoadBalancer, DomainError>> + Send;

    fn delete(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
}
