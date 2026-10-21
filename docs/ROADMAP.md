# Nodlance Roadmap

## Stage 0 — repository foundation

- [x] Initial Rust workspace and health/version API scaffold
- [x] PRD, architecture, threat model and contribution docs
- [x] CI workflow
- [x] Verify organization/repository name and publish repository
- [x] Add Cargo.lock from a successful, reviewed dependency resolution

## Stage 1 — inventory and enrollment

- [x] PostgreSQL metadata schema and migrations
- [x] User identity and RBAC
- [x] Host inventory and agent enrollment/revocation
- [x] Durable audit events

## Stage 2 — monitoring

- [x] Rust agent heartbeat and host metrics
- [x] PostgreSQL health and replication adapter
- [x] MySQL/MariaDB health and replication adapter
- [x] Metrics store integration and topology view
- [x] Alerts and maintenance windows

## Stage 3 — backup workflow

- [x] Persistent worker/scheduler and operation state machine
- [x] Backup adapters with explicit engine/version compatibility
- [x] Checksums, retention, restore drills and verification report

## Stage 4 — guarded provisioning and providers

- [x] Linux/VM provider
- [x] Docker provider
- [x] Kubernetes discovery/provider
- [x] Least-privilege installation and rollback workflow


## Stage 5 — recovery and HA

- [x] Operator-approved promotion workflow
- [x] Topology-specific fencing integration
- [x] Coordination/quorum and stale-command controls
- [x] Fault-injection qualification
- [x] Automatic failover only after documented safety gates
