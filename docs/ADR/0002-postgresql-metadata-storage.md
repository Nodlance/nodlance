# ADR 0002: PostgreSQL as control-plane metadata storage

- **Status:** Accepted for Stage 1
- **Date:** 2026-10-21

## Context

Stage 1 introduces persistent control-plane state: user identities, RBAC definitions, managed host inventories, node agent enrollment records, and append-only audit event trails. The storage system must provide strong ACID guarantees, robust indexing for query filters, native JSONB support for flexible metadata/labels, and battle-tested operational tooling.

## Decision

Use PostgreSQL (version 16+) as the authoritative metadata store for the Nodlance control plane. Manage database connections with SQLx connection pooling and execute schema evolution via embedded SQLx migrations on application startup.

Key architectural choices:
- UUIDv7 primary keys for time-ordered locality and optimal B-tree index performance.
- Native `INET` types for validated IP addresses and host routing.
- Append-only `audit_events` table for durable non-repudiation of privileged actions.
- Domain ports (`nodlance_domain::ports`) decoupled from SQLx database implementation (`nodlance-db`).

## Consequences

- Direct support for high-throughput, concurrent reads and writes with ACID transaction isolation.
- Embedded migrations eliminate external database migration tooling during deployment and local development.
- A running PostgreSQL instance is required for complete end-to-end integration environments (provided in `deploy/compose/compose.yaml`).
- Control-plane tests remain decoupled through repository abstraction traits and lazy connection configurations.
