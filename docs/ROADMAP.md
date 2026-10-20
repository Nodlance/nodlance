# Nodlance Roadmap

## Stage 0 — repository foundation

- [x] Initial Rust workspace and health/version API scaffold
- [x] PRD, architecture, threat model and contribution docs
- [x] CI workflow
- [x] Verify organization/repository name and publish repository
- [x] Add Cargo.lock from a successful, reviewed dependency resolution

## Stage 1 — inventory and enrollment

- [ ] PostgreSQL metadata schema and migrations
- [ ] User identity and RBAC
- [ ] Host inventory and agent enrollment/revocation
- [ ] Durable audit events

## Stage 2 — monitoring

- [ ] Rust agent heartbeat and host metrics
- [ ] PostgreSQL health and replication adapter
- [ ] MySQL/MariaDB health and replication adapter
- [ ] Metrics store integration and topology view
- [ ] Alerts and maintenance windows

## Stage 3 — backup workflow

- [ ] Persistent worker/scheduler and operation state machine
- [ ] Backup adapters with explicit engine/version compatibility
- [ ] Checksums, retention, restore drills and verification report

## Stage 4 — guarded provisioning and providers

- [ ] Linux/VM provider
- [ ] Docker provider
- [ ] Kubernetes discovery/provider
- [ ] Least-privilege installation and rollback workflow

## Stage 5 — recovery and HA

- [ ] Operator-approved promotion workflow
- [ ] Topology-specific fencing integration
- [ ] Coordination/quorum and stale-command controls
- [ ] Fault-injection qualification
- [ ] Automatic failover only after documented safety gates
