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

## Stage 6 — ClusterControl Web UI MVP (Severalnines Parity)

- [x] Embedded Axum 0.8 / `tower-http` static file server serving Nodlance ClusterControl at `/`
- [x] Real-time fleet overview dashboard with multi-engine filter (PostgreSQL, MySQL, MariaDB)
- [x] Interactive SVG replication topology graph with animated replication streams, lag badges, and node inspector
- [x] High Availability & Recovery Studio with candidate evaluator ranking, cryptographic operator approval token, and STONITH node fencing
- [x] 5-step guarded cluster deployment wizard with LIFO reverse rollback safety preview
- [x] Backups management and automated restore qualification drill verification reports (SHA-256 integrity)
- [x] Workload discovery scanner (Subnet CIDR / Docker / K8s) with one-click import
- [x] Host inventory and agent management with one-line installation command generator
- [x] Incident alarm acknowledgment flow and tamper-evident audit log trail

