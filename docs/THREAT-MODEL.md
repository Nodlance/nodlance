# Nodlance Threat Model (Initial)

## Assets

Database availability and integrity, database credentials, agent identities, backups, metadata, audit records, and control-plane integrity.

## Trust boundaries

1. API clients to control plane.
2. Control plane to node agents.
3. Agent to operating system and database server.
4. Control plane to discovered endpoints and external storage.
5. Maintainers/CI pipeline to release artifacts.

## Primary threats and controls

| Threat | Required controls |
|---|---|
| Agent impersonation | Per-agent identity, TLS/mTLS, revocation and rotation |
| Privilege escalation | Server-side authorization, scope checks, least privilege |
| Arbitrary command execution | Typed allowlisted operations; no arbitrary shell endpoint |
| SSRF through discovery | Endpoint validation, egress policy, blocking cloud metadata endpoints and disallowed ranges |
| Secret leakage | External secret references, structured-log redaction, access controls |
| Job replay/duplication | Idempotency keys, leases, attempt bounds, reconciliation |
| API denial of service | Rate/concurrency limits, timeouts, request limits and quotas |
| Backup tampering | Checksums, restricted storage permissions, encryption, restore tests |
| Audit tampering | Append-oriented event handling and separate protected export for high assurance |
| Split-brain | Topology-specific coordination, verifiable fencing, stale-command protection, routing and recovery tests |
| Supply-chain compromise | Dependency review, lockfile, CI checks, protected releases and provenance |

## Release blockers

- Any privileged endpoint without server-side authorization.
- Secrets appearing in logs or ordinary job payloads.
- Unbounded/retried side effects without idempotency or reconciliation.
- Automatic failover based only on heartbeats or a control-plane timeout.

## Current implementation status

The repository scaffold implements none of the database management operations above. The controls are design requirements for future implementation, not a claim of present security certification or production readiness.
