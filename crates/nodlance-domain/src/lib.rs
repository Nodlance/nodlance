//! Core Nodlance domain vocabulary.
//!
//! This crate intentionally avoids network, database, and web-framework dependencies.

pub mod agent;
pub mod audit;
pub mod engine;
pub mod error;
pub mod host;
pub mod ports;
pub mod user;

// Re-export commonly used types at the crate root.
pub use agent::{AgentIdentity, AgentStatus, EnrollmentRequest};
pub use audit::{AuditEvent, AuditFilter, NewAuditEvent};
pub use engine::{DatabaseEngine, JobStatus, NodeHealth, NodeRole};
pub use error::DomainError;
pub use host::{Host, HostFilter, HostStatus, NewHost, UpdateHost};
pub use ports::{AgentRepository, AuditRepository, HostRepository, UserRepository};
pub use user::{NewUser, Permission, Role, User};
