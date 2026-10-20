# Nodlance Technical Architecture

## 1. Architectural style

Start with a modular Rust control plane and a separately deployed node agent when agent operations are implemented. Avoid premature microservices. Enforce dependency direction through crates and traits.

## 2. Logical components

- **API:** HTTP routes, request validation, authentication/authorization middleware.
- **Application layer:** use cases, policy evaluation, orchestration interfaces.
- **Domain:** engine-neutral entities, state machines, capabilities, and error vocabulary.
- **Worker/scheduler:** durable jobs, lease/retry/reconciliation, introduced after metadata storage.
- **Node agent:** metrics and explicitly allowed typed operations; no arbitrary shell API.
- **Database adapters:** engine-specific health, metrics, replication and backup implementations.
- **Deployment providers:** Linux/bare metal, Docker, and Kubernetes integrations with scoped permissions.
- **Metadata store:** PostgreSQL for inventory, job state, alert metadata, configuration and audit metadata.
- **Metrics store:** Prometheus-compatible time-series store; do not use the metadata database as an unlimited metrics sink.
- **Secret provider:** secret references in metadata and jobs, with credentials held by a dedicated secret store or protected local mechanism.

## 3. Dependency direction

HTTP/API and worker layers depend on application contracts. Application use cases depend on domain types and ports. Engine/provider crates implement ports; domain must not depend on Axum, SQLx, network clients, or specific database drivers.

## 4. API and agent protocol

The current scaffold exposes only liveness, readiness, and version endpoints. Agent protocol and management endpoints must be versioned and authenticated before use. Prefer a simple HTTP/JSON contract initially; introduce gRPC only if concrete requirements justify it.

## 5. Durable job model

Each job should include an ID, tenant/scope, type, typed input schema, requester, idempotency key, status, lease/owner, deadline, bounded attempt count, audit correlation, and sanitized result/error. Network loss can leave an operation outcome uncertain; model `unknown_outcome` and `reconciliation_required` rather than blindly retrying every side-effecting command.

## 6. Health semantics

- Agent heartbeat answers whether the agent is communicating.
- Host metrics answer what the host currently reports.
- Database health checks answer whether a given database endpoint passes the configured checks.
- Replication status answers what the engine reports about replication at the time observed.

These are separate observations. None alone proves that a former primary is fenced and unable to accept writes.

## 7. High availability gate

Before automatic promotion, define the supported topology and demonstrate: reliable coordination/quorum appropriate to that topology; independently verifiable fencing of the old primary; candidate data-durability checks; stale command/epoch rejection; routing updates; and safe recovery/rejoin. Test network partitions, asymmetric partitions, agent crash, control-plane crash, delayed messages, and fencing failure. Automatic failover stays disabled until every gate is met.
