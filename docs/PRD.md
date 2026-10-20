# Nodlance — Product Requirements Document

**Status:** Draft / v0.1  
**Product type:** Open-source database operations platform  
**Initial engines:** PostgreSQL and MySQL/MariaDB  
**Deployment targets:** Linux bare metal/VM first; Docker and Kubernetes providers in stages

## 1. Vision

Provide a self-hosted control plane for discovering, monitoring, backing up, and safely operating database clusters across heterogeneous environments. Rust is the primary implementation language for the control plane and future node agent.

## 2. Personas

- Database administrators who inspect health, replication, backups, and maintenance operations.
- Platform/DevOps engineers who automate approved database workflows.
- System administrators who manage hosts and deployment providers.
- Security administrators who govern permissions, secret references, and audit evidence.
- Contributors who extend engine adapters, providers, integrations, and documentation.

## 3. MVP scope

### P0 — required

- Inventory for hosts, database clusters, and nodes.
- Secure enrollment identity for Linux node agents.
- Discovery and health checks for supported PostgreSQL and MySQL/MariaDB versions.
- Host metrics and database health observations.
- Replication observations and topology view; observed topology is not a failover authority.
- Alert rules, deduplication, acknowledgement, and maintenance windows.
- Durable job model, attempt history, timeouts, bounded retries, and audit events.
- Backup orchestration for explicitly tested engine/version combinations.
- Backup artifact metadata, checksums, retention policy, and documented restore validation.
- Local Docker Compose development setup and reproducible CI checks.

### P1 — after MVP

- Provisioning on explicitly supported Linux environments.
- Restore/PITR workflows with engine-specific adapters.
- Kubernetes discovery/provider.
- Manual, operator-approved recovery and promotion workflows with preconditions and reconciliation.

### P2 — advanced operations

- Automatic failover only after a topology-specific design demonstrates fencing, quorum/coordination, replication-state checks, stale-command prevention, routing changes, and recovery from network partitions.
- Rolling upgrades, scaling, more deployment providers, and community plugin interfaces.

## 4. Explicit non-goals for the initial scaffold

- Production database administration.
- Arbitrary remote shell execution.
- Automatic failover.
- Claiming support for an engine or provider solely because a domain enum exists.
- Storing raw database passwords, private keys, or access tokens in ordinary job payloads.

## 5. Non-functional goals

Initial test targets, subject to benchmark validation:

- 100 registered nodes in an integration-test environment.
- Default agent heartbeat interval of 15 seconds after the agent is implemented.
- A missed heartbeat produces `suspect`/`unknown` observation before any stronger conclusion.
- Persistent metadata for jobs and their state transitions.
- TLS/mTLS, least-privilege authorization, secret redaction, and auditable privileged operations.
- Documented engine/version compatibility matrix and tested restore procedures.

## 6. Acceptance criteria

- Supported database variants can be discovered and their supported health metrics viewed.
- Agent connectivity status is distinct from database health.
- A control-plane restart does not erase durable jobs or audit events.
- Retry/reconciliation behavior prevents uncontrolled duplicate side effects.
- Every privileged mutation is authorized and audited.
- Failed/corrupt backups are distinguishable from verified backups.
- No failover is triggered only because a heartbeat times out.
- Test and documentation claims match implemented capabilities.
