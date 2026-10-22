# Nodlance ClusterControl Web UI

Modern enterprise database operations console built with **Svelte 5** (`$state`, `$derived`, runes) and **Vite**. Provides 100% Severalnines ClusterControl operational parity with high-contrast Light and Dark theme support.

---

## Features

- **Fleet Overview**: Multi-engine cluster telemetry (PostgreSQL, MySQL, MariaDB) with live QPS, replication latency, and storage meters.
- **Replication Topology & Node Graph**: Interactive, dynamic SVG replication hierarchy with zero-overlap card layout, live LSN/GTID tracking, centered lag badges, and direct telemetry table integration.
- **Load Balancers & Connection Routers**: Full management and deployment of ProxySQL, PgBouncer, Keepalived VRRP VIP, and HAProxy with live configuration preview.
- **High Availability & Guarded Recovery**: Quorum monitoring, candidate evaluator scoring, cryptographic operator approval tokens, and STONITH node fencing.
- **Backups, PITR Streams & Sandbox Restore Drills**: Continuous Point-In-Time Recovery (PITR) WAL streaming, multi-cloud storage targets (AWS S3, MinIO, GCS, Azure Blob), and automated sandbox restore validation.
- **DBA Advisors & Rhai Script Engine**: Real-time rule evaluation (memory, connection pools, index health, replication lag), and slow query log analysis.
- **Config Management & PKI Certificate Authority**: Configuration template versioning with rolling diffs, plus automated X.509 mTLS Certificate Authority with 1-click renewal.
- **Guarded Provisioning Wizard**: 5-step deployment wizard with reverse LIFO rollback safety across Linux bare-metal, Docker, and Kubernetes.
- **Discovery Scanner**: Subnet CIDR and container probe to import existing unmanaged database workloads.
- **Host Inventory & Agent Management**: Enrolled nodes and copyable agent install command.
- **Alarms & Audit Trail**: Active alert acknowledgment and tamper-evident append-only security log.
- **Enterprise Design System**: Compliant with WCAG AA accessibility standards (minimum 4.5:1 contrast ratio) in both Light and Dark themes.

---

## Directory Structure

```text
ui/
├── public/                 # Static public assets (favicons, SVGs)
├── src/
│   ├── assets/             # Brand logos, icons, and SVG graphics
│   ├── components/
│   │   ├── TopNav.svelte             # Top navigation with cluster switcher, stats pills, theme toggle
│   │   ├── Sidebar.svelte            # Collapsible navigation sidebar with badge counters
│   │   ├── FleetView.svelte          # Fleet overview dashboard with engine filter pills
│   │   ├── TopologyView.svelte       # Interactive SVG topology graph with zero-overlap layout
│   │   ├── LoadBalancersView.svelte  # ProxySQL, PgBouncer, Keepalived & HAProxy management
│   │   ├── HaRecoveryView.svelte     # Quorum consensus, candidate scoring & failover
│   │   ├── BackupsView.svelte        # Multi-cloud backup schedules, PITR & restore drills
│   │   ├── AdvisorsView.svelte       # Rhai script advisor rules & slow query analyzer
│   │   ├── ConfigPkiView.svelte      # Configuration template diffs & X.509 mTLS CA
│   │   ├── ProvisioningView.svelte   # Guarded provisioning execution plans
│   │   ├── DiscoveryView.svelte      # Workload discovery scanner
│   │   ├── HostsView.svelte          # Infrastructure host inventory & heartbeat monitors
│   │   ├── AlarmsView.svelte         # Active alerts & notification acknowledgment
│   │   ├── AuditView.svelte          # Append-only security audit log
│   │   ├── Modals.svelte             # Enterprise modals (Deploy Wizard, Failover, Backup, Load Balancer)
│   │   └── Toast.svelte              # Non-blocking notification toasts
│   ├── lib/
│   │   ├── stores.js                 # Central Svelte stores (clusters, hosts, load balancers, theme)
│   │   └── data.js                   # Pre-seeded enterprise telemetry datasets
│   ├── app.css                       # Enterprise design system tokens, themes, and utilities
│   ├── App.svelte                    # Main root component and view router
│   └── main.js                       # Svelte application mounting entrypoint
├── package.json
└── vite.config.js
```

---

## Development

### Prerequisites

- **Node.js**: Version 20 or higher
- **npm**: Version 10 or higher

### Install Dependencies
```bash
npm install
```

### Start Development Server
```bash
npm run dev
```
The development server will start at `http://localhost:5173/` (or next available port).

### Build for Production
```bash
npm run build
```
Compiled production bundles will be written to `dist/`.

### Sync Assets to Backend API
To serve the frontend directly from the Axum backend (`crates/nodlance-api`):
```bash
cp -r dist/* ../crates/nodlance-api/static/
```
