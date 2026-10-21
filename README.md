# Nodlance

**Open-source database cluster operations platform**  
*Manage database fleets. Operate with confidence.*

Nodlance is an open-source platform providing a self-hosted control plane for discovering, monitoring, and safely operating database clusters across heterogeneous environments (Linux bare-metal, VMs, containers, and Kubernetes).

> **Status: Stage 1 — Inventory & Enrollment.** The control plane supports PostgreSQL metadata persistence, RBAC-protected authentication, host inventory management, agent enrollment/revocation, and append-only audit trails.

## Project principles

- Rust-first, modular architecture.
- Least privilege and explicit, typed operations; no arbitrary remote shell API.
- Observability before automation; automation before high availability.
- Database-engine differences remain explicit behind adapters.
- Persistent, auditable operations for mutations with side effects.
- No automatic failover until fencing, coordination/quorum, and recovery behavior are demonstrably safe for a specific topology.
- Community-led development with transparent decisions and contribution guidelines.

## Workspace structure

```text
nodlance/
├── crates/
│   ├── nodlance-domain/            # Shared domain entities, vocabulary, and repository ports
│   ├── nodlance-db/                # PostgreSQL storage, connection pool, migrations, repositories
│   ├── nodlance-auth/              # Argon2 password hashing, JWT tokens, RBAC context
│   ├── nodlance-adapter-postgres/  # PostgreSQL health, recovery, and replication probe adapter
│   ├── nodlance-adapter-mysql/     # MySQL/MariaDB health and replica status probe adapter
│   ├── nodlance-agent/             # Lightweight host monitoring daemon binary
│   └── nodlance-api/               # Axum HTTP API, middleware, route handlers
├── assets/                         # SVG icons, logo, and brand assets
├── docs/                           # PRD, architecture, threat model, ADRs, roadmap
├── openapi.yaml                    # OpenAPI 3.1 API specification
├── deploy/compose/                 # Docker Compose development environment (API + PostgreSQL)
└── .github/workflows/              # CI checks (fmt, clippy, tests)
```

## Requirements

- Rust stable toolchain (Rust 1.85 or later for declared edition 2024 / MSRV)
- Cargo
- PostgreSQL 16+ or Docker / Docker Compose

## Run locally

```bash
cp .env.example .env
cargo fmt --all --check
cargo test --workspace
cargo run -p nodlance-api
```

The API listens on `0.0.0.0:8080` by default. Override the port with `PORT`.

Verify endpoints:

```bash
curl http://localhost:8080/health/live
curl http://localhost:8080/health/ready
curl http://localhost:8080/api/v1/version
```

Authenticate and inspect hosts:

```bash
# 1. Login with initial admin credentials
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"adminpassword"}' | jq -r .token)

# 2. List hosts
curl -s http://localhost:8080/api/v1/hosts -H "Authorization: Bearer $TOKEN"
```

## Run with Docker Compose

```bash
docker compose -f deploy/compose/compose.yaml up --build
```

This starts PostgreSQL and the Nodlance API control plane with automatic database migrations and initial admin user seeding.

## API surface (Stage 1 & Stage 2)

| Method | Endpoint | Permission | Description |
|---|---|---|---|
| `GET` | `/health/live` | Public | Process liveness probe |
| `GET` | `/health/ready` | Public | Readiness probe (verifies DB connectivity) |
| `GET` | `/api/v1/version` | Public | Version and stage identifier |
| `POST` | `/api/v1/auth/login` | Public | Authenticate user, return JWT |
| `GET` | `/api/v1/users` | `user:read` | List users |
| `POST` | `/api/v1/users` | `user:write` | Create user |
| `GET` | `/api/v1/hosts` | `host:read` | List hosts (filter by status, name) |
| `POST` | `/api/v1/hosts` | `host:write` | Register host |
| `GET` | `/api/v1/hosts/{id}` | `host:read` | Get host details |
| `PUT` | `/api/v1/hosts/{id}` | `host:write` | Update host |
| `DELETE` | `/api/v1/hosts/{id}` | `host:write` | Decommission host |
| `GET` | `/api/v1/hosts/{id}/metrics` | `host:read` | Query host telemetry & metrics history |
| `POST` | `/api/v1/hosts/{id}/enroll` | `agent:write` | Enroll node agent identity |
| `POST` | `/api/v1/hosts/{id}/revoke` | `agent:write` | Revoke node agent identity |
| `POST` | `/api/v1/agent/heartbeat` | Agent | Ingest agent heartbeat and host metrics |
| `GET` | `/api/v1/clusters` | `host:read` | List database clusters |
| `POST` | `/api/v1/clusters` | `host:write` | Create database cluster |
| `GET` | `/api/v1/clusters/{id}` | `host:read` | Get cluster details |
| `POST` | `/api/v1/clusters/{id}/nodes` | `host:write` | Register database node into cluster |
| `GET` | `/api/v1/clusters/{id}/topology` | `host:read` | Observed cluster replication topology view |
| `GET` | `/api/v1/alerts/rules` | `audit:read` | List alert evaluation rules |
| `GET` | `/api/v1/alerts` | `audit:read` | List active alert incidents |
| `POST` | `/api/v1/alerts/{id}/acknowledge` | `host:write` | Acknowledge active alert incident |
| `POST` | `/api/v1/maintenance-windows` | `host:write` | Schedule maintenance window |
| `GET` | `/api/v1/audit` | `audit:read` | Query audit event log |

See [`openapi.yaml`](openapi.yaml) for full schema details.

## Roadmap

1. **Stage 0 — Foundation:** domain contracts, API, CI, design/security documentation.
2. **Stage 1 — Inventory & Enrollment:** PostgreSQL storage, RBAC, host inventory, agent enrollment, audit trail.
3. **Stage 2 — Monitoring:** agent heartbeat, host metrics, database health, replication topology.
4. **Stage 3 — Durable Jobs & Backup:** persistent queue, backup/restore drills, validation reports.
5. **Stage 4 — Provisioning:** guarded deployment providers for supported Linux environments.
6. **Stage 5 — Recovery & HA:** operator-approved promotion workflows; automatic failover after safety gates.

See [`docs/ROADMAP.md`](docs/ROADMAP.md) and [`docs/PRD.md`](docs/PRD.md).

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) before opening an issue or pull request. Architectural decisions belong in `docs/ADR/`.

## Security

See [`SECURITY.md`](SECURITY.md).

## License

Apache-2.0. See [`LICENSE`](LICENSE).
