//! Repository ports (traits) for domain entity persistence.

use uuid::Uuid;

use crate::{
    agent::AgentIdentity,
    audit::{AuditEvent, AuditFilter, NewAuditEvent},
    error::DomainError,
    host::{Host, HostFilter, NewHost, UpdateHost},
    user::{NewUser, Permission, Role, User},
};

/// Port for host inventory persistence operations.
pub trait HostRepository: Send + Sync {
    fn create(
        &self,
        created_by: Uuid,
        new_host: &NewHost,
    ) -> impl std::future::Future<Output = Result<Host, DomainError>> + Send;

    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<Host>, DomainError>> + Send;

    fn list(
        &self,
        filter: &HostFilter,
    ) -> impl std::future::Future<Output = Result<Vec<Host>, DomainError>> + Send;

    fn update(
        &self,
        id: Uuid,
        update: &UpdateHost,
    ) -> impl std::future::Future<Output = Result<Host, DomainError>> + Send;

    fn decommission(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<Host, DomainError>> + Send;
}

/// Port for agent enrollment and lifecycle persistence.
pub trait AgentRepository: Send + Sync {
    fn enroll(
        &self,
        host_id: Uuid,
        fingerprint: &str,
    ) -> impl std::future::Future<Output = Result<AgentIdentity, DomainError>> + Send;

    fn get_by_host_id(
        &self,
        host_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<AgentIdentity>, DomainError>> + Send;

    fn get_by_fingerprint(
        &self,
        fingerprint: &str,
    ) -> impl std::future::Future<Output = Result<Option<AgentIdentity>, DomainError>> + Send;

    fn revoke(
        &self,
        host_id: Uuid,
        revoked_by: Uuid,
    ) -> impl std::future::Future<Output = Result<AgentIdentity, DomainError>> + Send;
}

/// Port for user identity, role, and permission persistence.
pub trait UserRepository: Send + Sync {
    fn create(
        &self,
        new_user: &NewUser,
        password_hash: &str,
    ) -> impl std::future::Future<Output = Result<User, DomainError>> + Send;

    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<User>, DomainError>> + Send;

    fn get_by_username(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, DomainError>> + Send;

    fn list(&self) -> impl std::future::Future<Output = Result<Vec<User>, DomainError>> + Send;

    fn get_user_roles(
        &self,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Vec<Role>, DomainError>> + Send;

    fn get_user_permissions(
        &self,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Vec<Permission>, DomainError>> + Send;

    fn assign_role(
        &self,
        user_id: Uuid,
        role_name: &str,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;
}

/// Port for append-only audit event logging and querying.
pub trait AuditRepository: Send + Sync {
    fn record(
        &self,
        event: &NewAuditEvent,
    ) -> impl std::future::Future<Output = Result<AuditEvent, DomainError>> + Send;

    fn list(
        &self,
        filter: &AuditFilter,
    ) -> impl std::future::Future<Output = Result<Vec<AuditEvent>, DomainError>> + Send;
}
