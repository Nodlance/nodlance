# Nodlance

**Open-source database cluster operations platform**  
*Manage database fleets. Operate with confidence.*

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust: 1.85+](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Svelte: 5](https://img.shields.io/badge/Frontend-Svelte_5-red.svg)](https://svelte.dev/)
[![Docker: Ready](https://img.shields.io/badge/Docker-Compose_Ready-2496ED.svg)](deploy/compose/compose.yaml)
[![Status: Severalnines Parity Complete](https://img.shields.io/badge/Parity-Severalnines_ClusterControl-success.svg)](docs/CLUSTERCONTROL-PARITY-PLAN.md)

Nodlance is an enterprise-grade, open-source control plane for discovering, provisioning, monitoring, routing, and safely operating production database clusters across heterogeneous infrastructure (Linux bare-metal, virtual machines, Docker containers, and Kubernetes).

> **Status: Full Severalnines ClusterControl Parity Achieved (Milestones 1–6 Complete).**  
> Nodlance provides end-to-end multi-master Galera arbitration, high-throughput connection routing (ProxySQL, PgBouncer, Keepalived VRRP VIP, HAProxy), automated multi-cloud backups with Point-In-Time Recovery (PITR), programmable DBA Advisors powered by an embedded Rhai script engine, configuration template versioning with rolling diffs, an automated X.509 mTLS Certificate Authority, and a native Model Context Protocol (MCP) server for AI-assisted fleet operations.

---

## Key Capabilities

- **Multi-Engine Cluster Parity**: Native adapters for PostgreSQL 16+ (streaming replication, synchronous commit), MySQL 8.4 LTS (Group Replication, GTID), and MariaDB 11.4 Galera (wsrep quorum consensus, state transfers, and automatic `garbd` arbitrator daemon deployment).
- **Load Balancers & Connection Routers**: Automated provisioning, rule synchronization, and live configuration generation for:
  - **ProxySQL**: Query rule routing, read/write hostgroup splitting (10/20), and high-throughput connection multiplexing.
  - **PgBouncer**: Transaction-, session-, and statement-level connection pooling with SCRAM-SHA-256 authentication.
  - **Keepalived VRRP VIP**: Floating virtual IP failover with automated healthcheck scripts and zero application reconfiguration.
  - **HAProxy**: Layer-4 TCP stream load balancing with weighted round-robin and active backend health probes.
- **Continuous Backups & Multi-Cloud Retention**:
  - Full physical snapshots (`pg_basebackup`, Percona XtraBackup, MariaBackup) and continuous Point-In-Time Recovery (PITR) WAL streams.
  - Offloading to AWS S3, MinIO, Google Cloud Storage (GCS), Azure Blob Storage, and local ZFS pools with AES-256 encryption and Zstandard (`zstd`) compression.
  - Automated sandbox restore drills with SHA-256 checksum integrity verification.
- **DBA Advisors & Rhai Script Engine**:
  - Programmable rule engine executing safe, sandboxed Rhai scripts for performance inspection, buffer pool tuning, connection exhaustion prevention, indexing recommendations, and replication lag anomalies.
  - Real-time slow query log analyzer with P95/P99 latency categorization and index recommendation hints.
- **Configuration Management & Automated PKI CA**:
  - Version-controlled configuration templates (`postgresql.conf`, `my.cnf`, `wsrep.cnf`, `pg_hba.conf`) with side-by-side visual rolling diffs and rollback.
  - Built-in X.509 Certificate Authority issuing mTLS certificates, enforcing strict TLS 1.3, continuous expiry monitoring, and 1-click rotation.
- **Guarded Provisioning with LIFO Rollback**: 5-step guided provisioning wizard executing declarative plans across Linux systemd, Docker runtimes, or Kubernetes StatefulSets. If any step fails, Nodlance automatically executes reverse LIFO compensation to leave hosts untouched.
- **HA Recovery & Fencing Studio**: Quorum consensus validation, cryptographic operator approval tokens (Two-Man Rule), and STONITH fencing mechanisms to eliminate split-brain write hazards.
- **Model Context Protocol (MCP) Server**: Exposes fleet telemetry, cluster health, advisor checks, and guarded execution tools to AI assistants (Claude Desktop, Cursor, Windsurf, Antigravity) via `nodlance-mcp`.
- **Enterprise Svelte 5 Web UI**: Responsive control plane with high-contrast Light and Dark themes compliant with WCAG AA standards (4.5:1+ contrast ratio).

---

## Workspace Structure

The Nodlance codebase is organized as a unified, high-performance Rust workspace and Svelte 5 frontend:

```text
nodlance/
├── crates/
│   ├── nodlance-domain/            # Core entities, value objects, domain events, repository ports
│   ├── nodlance-db/                # PostgreSQL persistence, SQLx migrations, connection pooling
│   ├── nodlance-auth/              # Argon2id password hashing, JWT tokens, RBAC permission matrices
│   ├── nodlance-adapter-postgres/  # PostgreSQL 16 replication, slots, streaming, pg_basebackup
│   ├── nodlance-adapter-mysql/     # MySQL 8.4 Group Replication, GTID, XtraBackup adapter
│   ├── nodlance-adapter-galera/    # MariaDB Galera wsrep provider, SST/IST, garbd arbitrator unit
│   ├── nodlance-advisor/           # Rhai scripting engine, DBA rule evaluations, slow query analyzer
│   ├── nodlance-provider-linux/    # Guarded systemd service provisioning, preflight checks, rollback
│   ├── nodlance-provider-docker/   # Guarded Docker bridge network & containerized database deployment
│   ├── nodlance-provider-k8s/      # Kubernetes StatefulSet and Headless service declarative controller
│   ├── nodlance-recovery/          # Fencing engine (STONITH), quorum consensus, epoch monotonicity
│   ├── nodlance-agent/             # Lightweight host daemon reporting CPU/RAM/disk and node telemetry
│   ├── nodlance-worker/            # Leased background worker for durable backup, restore, and failover
│   ├── nodlance-mcp/               # Model Context Protocol (MCP) server for AI assistant DBAs
│   ├── nodlance-cli/               # CLI utility for fleet inspection, backup triggers, and health checks
│   └── nodlance-api/               # Axum HTTP control plane API, OpenAPI specs, static UI serving
├── ui/                             # Enterprise Svelte 5 + Vite Web UI (ClusterControl parity)
├── deploy/compose/                 # Docker Compose development & production deployment stack
├── docs/                           # Architecture specs, PRD, ADRs, Threat Model, Parity Plan
├── openapi.yaml                    # OpenAPI 3.1 REST API specification
└── Dockerfile                      # Optimized multi-stage Docker build with stripped release binaries
```

---

## Quickstart

### Prerequisites

- **Rust**: Version 1.85+ (Edition 2024 / MSRV)
- **Node.js**: Version 20+ (for Svelte UI compilation)
- **Docker & Docker Compose**: Recommended for local PostgreSQL metadata persistence

---

### Option A: Run with Docker Compose (Recommended)

Start the entire Nodlance Control Plane (API + PostgreSQL backend) with a single command:

```bash
docker compose -f deploy/compose/compose.yaml up --build
```

The service will start and bind to `http://localhost:8090` (configurable via `PORT`):

- **ClusterControl Web UI**: [http://localhost:8090/](http://localhost:8090/)
- **Liveness Health Check**: `http://localhost:8090/health/live`
- **Readiness Health Check**: `http://localhost:8090/health/ready`
- **API Version & Parity Stage**: `http://localhost:8090/api/v1/version`

Default administrative credentials seeded on first startup:
- **Username**: `admin`
- **Password**: `adminpassword`

---

### Option B: Build and Run Locally from Source

#### 1. Setup Environment
```bash
cp .env.example .env
```

#### 2. Build the Svelte 5 Frontend
```bash
cd ui
npm install
npm run build
cd ..
cp -r ui/dist/* crates/nodlance-api/static/
```

#### 3. Compile and Run the Rust Control Plane
```bash
# Verify formatting, linting, and workspace test suite
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace

# Start the Axum HTTP API
PORT=8090 cargo run -p nodlance-api
```

---

## Nodlance ClusterControl Web UI

Access the Web UI at `http://localhost:8090/`. It includes 10 purpose-built operational studios:

| View | Description | Key Features |
|---|---|---|
| **Fleet Overview** | High-level fleet telemetry across all managed engines | Aggregate QPS, replication latency SLA meters, storage usage, engine filter pills |
| **Topology & Nodes** | Interactive replication hierarchy graph | Zero-overlap dynamic SVG layout, live LSN/GTID tracking, lag badges, telemetry table |
| **Load Balancers** | Connection routing and pooling management | ProxySQL, PgBouncer, Keepalived VRRP VIP, HAProxy cards, live config modal, `garbd` arbitrator |
| **HA & Recovery** | Automated failover and fencing studio | Quorum voting consensus, candidate scoring, cryptographic approval tokens, STONITH triggers |
| **Backups & Drills** | Disaster recovery & continuous PITR | S3/MinIO/GCS cloud schedules, continuous WAL rewind streams, automated restore drill reports |
| **DBA Advisors** | Automated health checks & Rhai scripts | Memory/pool/index rules, custom Rhai script editor, slow query analyzer with P95/P99 latency |
| **Config & PKI CA** | Configuration management & X.509 mTLS | Parameter template versioning with visual diffs, built-in CA root, 1-click certificate renewal |
| **Guarded Provision** | Safe infrastructure database deployments | 5-step wizard supporting Linux, Docker, and K8s with reverse LIFO rollback verification |
| **Discovery Scanner** | Network & container scanner | CIDR subnet probe (5432, 3306, 4567) to import unmanaged existing database instances |
| **Host Inventory** | Registered infrastructure nodes & agents | Heartbeat monitors, CPU/RAM meters, and copyable 1-line agent enrollment curl script |

---

## API Surface

Nodlance exposes a fully documented OpenAPI 3.1 REST API:

### Authentication & System Health
| Method | Endpoint | Permission | Description |
|---|---|---|---|
| `GET` | `/health/live` | Public | Process liveness probe |
| `GET` | `/health/ready` | Public | Database readiness probe |
| `GET` | `/api/v1/version` | Public | System version & parity stage |
| `POST` | `/api/v1/auth/login` | Public | Authenticate user, return JWT |
| `GET` | `/api/v1/users` | `user:read` | List registered operators |
| `POST` | `/api/v1/users` | `user:write` | Create operator account |

### Fleet & Replication Topology
| Method | Endpoint | Permission | Description |
|---|---|---|---|
| `GET` | `/api/v1/clusters` | `host:read` | List all managed database clusters |
| `POST` | `/api/v1/clusters` | `host:write` | Register new database cluster |
| `GET` | `/api/v1/clusters/{id}` | `host:read` | Inspect cluster telemetry & metadata |
| `POST` | `/api/v1/clusters/{id}/nodes` | `host:write` | Add node to database cluster |
| `GET` | `/api/v1/clusters/{id}/topology` | `host:read` | Live replication topology hierarchy |

### Load Balancing & Connection Routers
| Method | Endpoint | Permission | Description |
|---|---|---|---|
| `GET` | `/api/v1/load-balancers` | `host:read` | List all routers (ProxySQL, PgBouncer, Keepalived, HAProxy) |
| `POST` | `/api/v1/load-balancers` | `host:write` | Deploy new connection router or VIP daemon |
| `POST` | `/api/v1/load-balancers/{id}/reload` | `host:write` | Synchronize query rules & reload runtime config |
| `GET` | `/api/v1/load-balancers/{id}/config` | `host:read` | Retrieve generated live router configuration |

### Galera Cluster & Multi-Master Arbitration
| Method | Endpoint | Permission | Description |
|---|---|---|---|
| `GET` | `/api/v1/galera/clusters/{id}/status` | `host:read` | Query wsrep cluster state, primary component & UUID |
| `POST` | `/api/v1/galera/clusters/{id}/arbitrator` | `host:write` | Generate lightweight `garbd` arbitrator daemon unit |
| `POST` | `/api/v1/galera/nodes/{id}/desync` | `host:write` | Desync node for non-blocking backup or maintenance |

### High Availability, Quorum & Fencing
| Method | Endpoint | Permission | Description |
|---|---|---|---|
| `GET` | `/api/v1/clusters/{id}/recovery/candidates` | `host:read` | Evaluated replica candidate rankings |
| `POST` | `/api/v1/clusters/{id}/recovery/promote` | `host:write` | Request operator-approved replica promotion |
| `POST` | `/api/v1/clusters/{id}/recovery/promote/{req_id}/approve` | `host:write` | Approve promotion with cryptographic token |
| `POST` | `/api/v1/clusters/{id}/recovery/fence` | `host:write` | Trigger STONITH node isolation |
| `GET` | `/api/v1/clusters/{id}/recovery/epoch` | `host:read` | Get cluster epoch status & term history |
| `POST` | `/api/v1/recovery/fault-injection` | `host:write` | Run automated chaos & failover qualification tests |

### Backups, PITR Streams & Restore Drills
| Method | Endpoint | Permission | Description |
|---|---|---|---|
| `GET` | `/api/v1/clusters/{id}/backups` | `host:read` | List backup snapshot artifacts |
| `POST` | `/api/v1/clusters/{id}/backups` | `host:write` | Trigger immediate physical backup |
| `POST` | `/api/v1/backup-schedules` | `host:write` | Create automated cron backup schedule (S3, MinIO, GCS, Azure) |
| `GET` | `/api/v1/backup-schedules` | `host:read` | List active automated backup schedules |
| `DELETE` | `/api/v1/backup-schedules/{id}` | `host:write` | Remove automated backup schedule |
| `GET` | `/api/v1/pitr/streams` | `host:read` | List continuous Point-In-Time Recovery WAL streams |
| `POST` | `/api/v1/pitr/streams/{id}/rewind` | `host:write` | Execute continuous PITR stream rewind to timestamp |
| `POST` | `/api/v1/backups/{id}/verify` | `host:write` | Execute sandbox restore validation drill |
| `GET` | `/api/v1/backups/{id}/drills/latest` | `host:read` | Get latest restore drill report and checksum verification |

### DBA Advisors & Performance Tuning
| Method | Endpoint | Permission | Description |
|---|---|---|---|
| `GET` | `/api/v1/advisors/rules` | `audit:read` | List registered Rhai script advisor rules |
| `POST` | `/api/v1/advisors/rules` | `host:write` | Register custom Rhai DBA evaluation script |
| `GET` | `/api/v1/advisors/findings` | `audit:read` | List active performance & security findings |
| `GET` | `/api/v1/advisors/slow-queries` | `audit:read` | Query top slow queries with execution telemetry |

### Configuration Templates & PKI CA
| Method | Endpoint | Permission | Description |
|---|---|---|---|
| `GET` | `/api/v1/config/templates` | `host:read` | List database configuration templates |
| `POST` | `/api/v1/config/templates` | `host:write` | Save new configuration template version |
| `POST` | `/api/v1/config/diff` | `host:read` | Generate side-by-side rolling configuration diff |
| `GET` | `/api/v1/pki/certificates` | `host:read` | List active X.509 mTLS certificates |
| `POST` | `/api/v1/pki/certificates/renew` | `host:write` | Issue or renew node mTLS certificate |

### Guarded Provisioning & Workload Discovery
| Method | Endpoint | Permission | Description |
|---|---|---|---|
| `POST` | `/api/v1/provisioning/plans` | `host:write` | Generate ordered guarded deployment plan |
| `GET` | `/api/v1/provisioning/plans/{id}` | `host:read` | Inspect deployment plan steps and status |
| `POST` | `/api/v1/provisioning/plans/{id}/execute` | `host:write` | Enqueue guarded execution |
| `POST` | `/api/v1/provisioning/plans/{id}/rollback` | `host:write` | Trigger automated reverse LIFO rollback |
| `POST` | `/api/v1/discovery/scan` | `host:write` | Run CIDR subnet scan for unmanaged database ports |
| `GET` | `/api/v1/discovery/workloads` | `host:read` | List discovered candidate workloads |

### Infrastructure Hosts & Agents
| Method | Endpoint | Permission | Description |
|---|---|---|---|
| `GET` | `/api/v1/hosts` | `host:read` | List registered infrastructure hosts |
| `POST` | `/api/v1/hosts` | `host:write` | Register new infrastructure host |
| `POST` | `/api/v1/hosts/{id}/enroll` | `agent:write` | Enroll host agent identity |
| `POST` | `/api/v1/hosts/{id}/revoke` | `agent:write` | Revoke agent identity |
| `POST` | `/api/v1/agent/heartbeat` | Agent | Ingest agent heartbeat and resource metrics |
| `GET` | `/api/v1/jobs` | `audit:read` | List background worker execution jobs |
| `GET` | `/api/v1/alerts` | `audit:read` | List active alert incidents |
| `POST` | `/api/v1/alerts/{id}/acknowledge` | `host:write` | Acknowledge active alert |
| `GET` | `/api/v1/audit` | `audit:read` | Query append-only audit event log |

---

## Model Context Protocol (MCP) Server

Nodlance includes a native Model Context Protocol (MCP) server in [`crates/nodlance-mcp/`](crates/nodlance-mcp), allowing AI assistants to query cluster health, evaluate DBA advisor rules, inspect slow queries, and trigger safe operations.

### Configuration for Claude Desktop / Cursor / Antigravity

Add the following entry to your `mcp_config.json`:

```json
{
  "mcpServers": {
    "nodlance": {
      "command": "cargo",
      "args": ["run", "-p", "nodlance-mcp", "--quiet"],
      "env": {
        "NODLANCE_API_URL": "http://localhost:8090",
        "NODLANCE_TOKEN": "your-jwt-auth-token"
      }
    }
  }
}
```

---

## Contributing

We welcome community contributions! Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) before submitting pull requests.

Ensure all checks pass before submitting:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

---

## Security

For vulnerability disclosure and security reporting guidelines, see [`SECURITY.md`](SECURITY.md).

---

## License

Nodlance is open-source software licensed under the **Apache License, Version 2.0**. See [`LICENSE`](LICENSE) for details.
