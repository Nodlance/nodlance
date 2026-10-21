//! Core Nodlance domain vocabulary.
//!
//! This crate intentionally avoids network, database, and web-framework dependencies.

pub mod agent;
pub mod alert;
pub mod audit;
pub mod cluster;
pub mod engine;
pub mod error;
pub mod heartbeat;
pub mod host;
pub mod metrics;
pub mod observation;
pub mod ports;
pub mod user;

// Re-export commonly used types at the crate root.
pub use agent::{AgentIdentity, AgentStatus, EnrollmentRequest};
pub use alert::{
    AlertIncident, AlertRule, AlertSeverity, AlertStatus, MaintenanceWindow, NewMaintenanceWindow,
};
pub use audit::{AuditEvent, AuditFilter, NewAuditEvent};
pub use cluster::{Cluster, DatabaseNode, NewCluster, NewDatabaseNode};
pub use engine::{DatabaseEngine, JobStatus, NodeHealth, NodeRole};
pub use error::DomainError;
pub use heartbeat::{AgentHeartbeat, HeartbeatPayload};
pub use host::{Host, HostFilter, HostStatus, NewHost, UpdateHost};
pub use metrics::HostMetrics;
pub use observation::{ClusterTopology, NodeObservation};
pub use ports::{
    AgentRepository, AlertRepository, AuditRepository, ClusterRepository, HeartbeatRepository,
    HostRepository, ObservationRepository, UserRepository,
};
pub use user::{NewUser, Permission, Role, User};
