//! Core Nodlance domain vocabulary.
//!
//! This crate intentionally avoids network, database, and web-framework dependencies.

pub mod advisor;
pub mod agent;
pub mod alert;
pub mod audit;
pub mod backup;
pub mod backup_schedule;
pub mod cluster;
pub mod config_pki;
pub mod engine;
pub mod error;
pub mod galera;
pub mod heartbeat;
pub mod host;
pub mod job;
pub mod load_balancer;
pub mod metrics;
pub mod observation;
pub mod ports;
pub mod provisioning;
pub mod recovery;
pub mod user;

// Re-export commonly used types at the crate root.
pub use advisor::{AdvisorCategory, AdvisorFinding, AdvisorRule, AdvisorSeverity, QueryDigest};
pub use agent::{AgentIdentity, AgentStatus, EnrollmentRequest};
pub use alert::{
    AlertIncident, AlertRule, AlertSeverity, AlertStatus, MaintenanceWindow, NewMaintenanceWindow,
};
pub use audit::{AuditEvent, AuditFilter, NewAuditEvent};
pub use backup::{
    BackupArtifact, BackupStatus, BackupType, NewBackupArtifact, RestoreValidationDrill,
    RetentionPolicy,
};
pub use backup_schedule::{
    BackupSchedule, CompressionFormat, NewBackupSchedule, PitrStream, StorageTarget,
    UpdateBackupSchedule,
};
pub use cluster::{Cluster, DatabaseNode, NewCluster, NewDatabaseNode};
pub use config_pki::{
    CertType, ConfigDiff, ConfigTemplate, IssuedCertificate, PkiCertificate, calculate_config_diff,
    parameter_requires_restart,
};
pub use engine::{DatabaseEngine, JobStatus, NodeHealth, NodeRole};
pub use error::DomainError;
pub use galera::{
    GaleraArbitratorConfig, GaleraObservation, SstMethod, WsrepClusterStatus, WsrepLocalState,
};
pub use heartbeat::{AgentHeartbeat, HeartbeatPayload};
pub use host::{Host, HostFilter, HostStatus, NewHost, UpdateHost};
pub use job::{Job, JobAttempt, JobType, NewJob};
pub use load_balancer::{
    HaProxyConfig, KeepalivedConfig, LoadBalancer, LoadBalancerConfigSnapshot, LoadBalancerFilter,
    LoadBalancerStatus, LoadBalancerType, NewLoadBalancer, PgBouncerConfig, PgBouncerPoolMode,
    ProxySqlBackendServer, ProxySqlConfig, ProxySqlQueryRule, UpdateLoadBalancer,
};
pub use metrics::HostMetrics;
pub use observation::{ClusterTopology, NodeObservation};
pub use ports::{
    AdvisorRepository, AgentRepository, AlertRepository, AuditRepository, BackupRepository,
    BackupScheduleRepository, ClusterRepository, ConfigurationRepository, DiscoveryRepository,
    FencingProvider, HeartbeatRepository, HostRepository, JobRepository, LoadBalancerRepository,
    ObservationRepository, PkiRepository, ProvisioningRepository, RecoveryRepository,
    UserRepository,
};
pub use provisioning::{
    DiscoveredWorkload, NewDiscoveredWorkload, NewProvisioningPlan, ProviderType,
    ProvisioningConfig, ProvisioningPlan, ProvisioningStatus, ProvisioningStep,
    ProvisioningStepType, StepStatus,
};
pub use recovery::{
    ClusterEpoch, FaultInjectionResult, FaultInjectionScenario, FencingExecution, FencingStatus,
    FencingType, NewPromotionRequest, PromotionCandidate, PromotionRequest, PromotionSafetyGates,
    PromotionStatus, UpdatePromotionParams,
};
pub use user::{NewUser, Permission, Role, User};
