# Nodlance

**Open-source database cluster operations platform**  
*Manage database fleets. Operate with confidence.*

Nodlance is an early-stage project to build a self-hosted control plane for discovering, monitoring, and eventually automating operations across PostgreSQL and MySQL/MariaDB deployments on Linux hosts, VMs, containers, and Kubernetes.

> **Status: scaffold / pre-alpha.** This repository currently contains a small Rust workspace and a health/version API. It does not yet connect to real databases, persist metadata, run an agent, create backups, provision servers, or perform failover. Do not use it to manage production infrastructure.

## Project principles

- Rust-first, modular architecture.
- Least privilege and explicit, typed operations; no arbitrary remote shell API.
- Observability before automation; automation before high availability.
- Database-engine differences remain explicit behind adapters.
- Persistent, auditable jobs for operations with side effects.
- No automatic failover until fencing, coordination/quorum, and recovery behavior are demonstrably safe for a specific topology.
- Community-led development with transparent decisions and contribution guidelines.

## Current workspace

```text
nodlance/
├── crates/
│   ├── nodlance-domain/    # Shared domain types; no I/O or framework dependencies
│   └── nodlance-api/       # Initial HTTP API and health endpoints
├── docs/                   # PRD, architecture, threat model, ADRs, roadmap
├── openapi.yaml            # Initial API contract
├── deploy/compose/         # Local development container
└── .github/workflows/      # CI checks
```

Future components such as the node agent, job worker, metadata repository, and database/provider adapters should be added when their contracts and tests are ready; they are intentionally not represented as working components yet.

## Requirements

- Rust stable toolchain (Rust 1.85 or later for the declared edition/MSRV)
- Cargo
- Docker and Docker Compose (optional, for containerized API development)

## Run locally

```bash
cp .env.example .env
cargo fmt --all --check
cargo test --workspace
cargo run -p nodlance-api
```

The API listens on `0.0.0.0:8080` by default. Override the port with `PORT`.

```bash
curl http://localhost:8080/health/live
curl http://localhost:8080/health/ready
curl http://localhost:8080/api/v1/version
```

## Run with Docker Compose

```bash
docker compose -f deploy/compose/compose.yaml up --build
```

Then open `http://localhost:8080/health/live`.

## API endpoints in this scaffold

| Method | Endpoint | Purpose |
|---|---|---|
| `GET` | `/health/live` | Process liveness |
| `GET` | `/health/ready` | Readiness of the current, dependency-free scaffold |
| `GET` | `/api/v1/version` | Name and version |

The readiness endpoint must be extended to check configured dependencies when metadata storage and other runtime dependencies are implemented. It must not report ready when required dependencies are unavailable.

## Roadmap

1. **Foundation:** domain contracts, API, CI, design/security documentation.
2. **Inventory:** registered hosts, enrollment identity, database discovery.
3. **Monitoring:** heartbeat, host metrics, database health, replication observations, topology UI.
4. **Durable jobs and backup:** persistent queue, audit events, backup and restore validation.
5. **Provisioning:** guarded deployment providers for supported environments.
6. **High availability:** first operator-approved recovery workflows; automatic failover only after explicit fencing and coordination safety gates.

See [`docs/ROADMAP.md`](docs/ROADMAP.md) and [`docs/PRD.md`](docs/PRD.md).

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) before opening an issue or pull request. Architectural decisions belong in `docs/ADR/`. Please do not submit unsafe automation shortcuts or claim support for an engine/provider without integration tests.

## Security

See [`SECURITY.md`](SECURITY.md). Do not report vulnerabilities in public issues.

## License

Proposed project license: Apache-2.0. See [`LICENSE`](LICENSE). Confirm the license choice and project ownership policy before the first public release.
