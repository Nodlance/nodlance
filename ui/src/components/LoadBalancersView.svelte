<script>
  import { loadBalancers, reloadLoadBalancer, openModal, showToast, clusters } from '../lib/stores.js';

  let selectedType = $state('all');
  let selectedLbModal = $state(null);
  let showConfigModal = $state(false);
  let generatedConfigText = $state('');

  let filteredLbs = $derived(
    selectedType === 'all'
      ? $loadBalancers
      : $loadBalancers.filter(lb => lb.type === selectedType)
  );

  function handleReload(lb) {
    reloadLoadBalancer(lb.id);
  }

  function viewConfig(lb) {
    if (lb.type === 'proxysql') {
      generatedConfigText = `-- ProxySQL SQLite Admin Commands (Port ${lb.adminPort || 6032})
DELETE FROM mysql_servers;
INSERT INTO mysql_servers (hostgroup_id, hostname, port, weight, max_connections) 
  VALUES (10, '${lb.ip}', 3306, 100, 1000);
INSERT INTO mysql_servers (hostgroup_id, hostname, port, weight, max_connections) 
  VALUES (20, '${lb.ip.replace(/\\d+$/, '16')}', 3306, 100, 1000);

DELETE FROM mysql_query_rules;
INSERT INTO mysql_query_rules (rule_id, match_pattern, destination_hostgroup, active)
  VALUES (1, '^SELECT.*FOR UPDATE', 10, 1);
INSERT INTO mysql_query_rules (rule_id, match_pattern, destination_hostgroup, active)
  VALUES (2, '^SELECT', 20, 1);

LOAD MYSQL SERVERS TO RUNTIME;
SAVE MYSQL SERVERS TO DISK;
LOAD MYSQL QUERY RULES TO RUNTIME;
SAVE MYSQL QUERY RULES TO DISK;`;
    } else if (lb.type === 'pgbouncer') {
      generatedConfigText = `; PgBouncer Configuration (Port ${lb.listenPort})
[databases]
* = host=${lb.ip} port=5432 auth_user=postgres

[pgbouncer]
listen_port = ${lb.listenPort}
listen_addr = *
auth_type = scram-sha-256
auth_file = /etc/pgbouncer/userlist.txt
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 50
min_pool_size = 10
reserve_pool_size = 5
reserve_pool_timeout = 5
server_idle_timeout = 600`;
    } else {
      generatedConfigText = `! Keepalived VRRP Configuration
global_defs {
    router_id nodlance_vrrp_router
    enable_script_security
}

vrrp_script chk_db {
    script "/usr/local/bin/nodlance_check_primary.sh"
    interval 2
    weight 2
}

vrrp_instance VI_51 {
    state MASTER
    interface eth0
    virtual_router_id 51
    priority 101
    advert_int 1
    virtual_ipaddress {
        192.168.10.100/24
    }
    track_script {
        chk_db
    }
}`;
    }
    selectedLbModal = lb;
    showConfigModal = true;
  }
</script>

<div class="view-container">
  <div class="view-header">
    <div>
      <h2 class="view-title">Load Balancers & Connection Routers</h2>
      <p class="view-subtitle">Severalnines ClusterControl Parity • ProxySQL, PgBouncer, HAProxy, and Keepalived VRRP VIP</p>
    </div>
    <div class="header-actions">
      <button class="btn btn-primary" onclick={() => openModal('deployLoadBalancer')}>
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        Deploy Router / VIP
      </button>
    </div>
  </div>

  <!-- Summary KPI Cards -->
  <div class="stats-row">
    <div class="stat-card">
      <div class="stat-label">Active Routers</div>
      <div class="stat-value">{$loadBalancers.length}</div>
      <div class="stat-trend trend-up">ProxySQL & PgBouncer Active</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Total Routed QPS</div>
      <div class="stat-value">30,550</div>
      <div class="stat-trend trend-up">Sub-millisecond query routing</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Virtual IP (VRRP)</div>
      <div class="stat-value">192.168.10.100</div>
      <div class="stat-trend trend-up">Master (Node 01)</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Failover Protection</div>
      <div class="stat-value">Automated</div>
      <div class="stat-trend trend-up">Synced with STONITH Fencing</div>
    </div>
  </div>

  <!-- Filter tabs -->
  <div class="filter-bar">
    <div class="filter-chips">
      <button class="filter-chip {selectedType === 'all' ? 'active' : ''}" onclick={() => selectedType = 'all'}>
        All Instances ({$loadBalancers.length})
      </button>
      <button class="filter-chip {selectedType === 'proxysql' ? 'active' : ''}" onclick={() => selectedType = 'proxysql'}>
        ProxySQL
      </button>
      <button class="filter-chip {selectedType === 'pgbouncer' ? 'active' : ''}" onclick={() => selectedType = 'pgbouncer'}>
        PgBouncer
      </button>
      <button class="filter-chip {selectedType === 'keepalived' ? 'active' : ''}" onclick={() => selectedType = 'keepalived'}>
        Keepalived VIP
      </button>
    </div>
  </div>

  <!-- Load Balancer Cards Grid -->
  <div class="lb-grid">
    {#each filteredLbs as lb (lb.id)}
      <div class="lb-card">
        <div class="lb-card-header">
          <div class="lb-brand">
            <span class="lb-badge lb-badge-{lb.type}">{lb.type.toUpperCase()}</span>
            <span class="lb-name">{lb.name}</span>
          </div>
          <span class="badge badge-success">
            <span class="pulse-indicator"></span> {lb.status.toUpperCase()}
          </span>
        </div>

        <div class="lb-card-body">
          <div class="lb-prop-row">
            <span class="lb-prop-label">Target Cluster:</span>
            <span class="lb-prop-value fw-600">{lb.clusterName}</span>
          </div>
          <div class="lb-prop-row">
            <span class="lb-prop-label">Host Node:</span>
            <span class="lb-prop-value font-mono">{lb.host} ({lb.ip})</span>
          </div>
          <div class="lb-prop-row">
            <span class="lb-prop-label">Routing Mode:</span>
            <span class="lb-prop-value text-accent">{lb.mode}</span>
          </div>
          <div class="lb-prop-row">
            <span class="lb-prop-label">Ports:</span>
            <span class="lb-prop-value font-mono">
              Listen: <span class="text-highlight">{lb.listenPort}</span>
              {#if lb.adminPort} • Admin: {lb.adminPort}{/if}
            </span>
          </div>
          <div class="lb-prop-row">
            <span class="lb-prop-label">Backends:</span>
            <span class="lb-prop-value font-mono text-muted">{lb.writers} | {lb.readers}</span>
          </div>

          <div class="lb-metrics-strip">
            <div class="metric-item">
              <span class="metric-num">{lb.qps.toLocaleString()}</span>
              <span class="metric-sub">QPS</span>
            </div>
            <div class="metric-item">
              <span class="metric-num">{lb.activeConnections}</span>
              <span class="metric-sub">Connections</span>
            </div>
            <div class="metric-item">
              <span class="metric-num">{lb.cacheHitRatio}</span>
              <span class="metric-sub">Cache / Multiplex</span>
            </div>
          </div>
        </div>

        <div class="lb-card-footer">
          <button class="btn btn-secondary btn-sm" onclick={() => viewConfig(lb)}>
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
            View Config
          </button>
          <button class="btn btn-primary btn-sm" onclick={() => handleReload(lb)}>
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 0 1 15-6.7L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-15 6.7L3 16"/><path d="M3 21v-5h5"/></svg>
            Sync Rules
          </button>
        </div>
      </div>
    {/each}
  </div>

  <!-- Galera Quorum & Arbitrator Notice Banner -->
  <div class="galera-arbitrator-banner">
    <div class="banner-icon">
      <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
    </div>
    <div class="banner-content">
      <div class="banner-title">Galera Cluster Arbitrator (garbd) Parity Engine</div>
      <p class="banner-text">
        ClusterControl recommends deploying a lightweight non-storage Garbd arbitrator whenever a Galera cluster has an even number of nodes to ensure reliable odd-node majority quorum and eliminate split-brain isolation risks.
      </p>
    </div>
    <div class="banner-action">
      <button class="btn btn-secondary" onclick={() => {
        showToast('Generated garbd arbitrator unit for core-mysql-ecommerce (odd-node quorum secured)', 'success');
      }}>
        Generate Garbd Daemon Unit
      </button>
    </div>
  </div>
</div>

<!-- Modal for Viewing Generated Configuration -->
{#if showConfigModal && selectedLbModal}
  <div class="modal-backdrop">
    <div class="modal-card">
      <div class="modal-header">
        <h3 class="modal-title">Live Generated Configuration: {selectedLbModal.name}</h3>
        <button class="modal-close" onclick={() => showConfigModal = false}>&times;</button>
      </div>
      <div class="modal-body">
        <p class="text-muted mb-3">
          This configuration is automatically generated by Nodlance based on cluster topology, read/write splitting rules, and node health.
        </p>
        <pre class="config-code-block">{generatedConfigText}</pre>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={() => {
          navigator.clipboard.writeText(generatedConfigText);
          showToast('Configuration copied to clipboard!', 'success');
        }}>
          Copy to Clipboard
        </button>
        <button class="btn btn-primary" onclick={() => showConfigModal = false}>
          Close
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .lb-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
    gap: 1.25rem;
    margin-bottom: 2rem;
  }

  .lb-card {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .lb-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
  }

  .lb-card-header {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--bg-surface);
  }

  .lb-brand {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .lb-badge {
    font-size: 0.72rem;
    font-weight: 700;
    padding: 0.2rem 0.5rem;
    border-radius: var(--radius-sm);
    letter-spacing: 0.05em;
  }

  .lb-badge-proxysql {
    background: #e0f2fe;
    color: #0369a1;
  }

  .lb-badge-pgbouncer {
    background: #ede9fe;
    color: #6d28d9;
  }

  .lb-badge-keepalived {
    background: #fef3c7;
    color: #b45309;
  }

  .lb-name {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 0.98rem;
  }

  .lb-card-body {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    flex: 1;
  }

  .lb-prop-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.85rem;
  }

  .lb-prop-label {
    color: var(--text-muted);
  }

  .lb-prop-value {
    color: var(--text-primary);
  }

  .text-accent {
    color: var(--color-primary);
    font-weight: 500;
  }

  .text-highlight {
    font-weight: 700;
    color: var(--color-primary);
  }

  .lb-metrics-strip {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px dashed var(--border-color);
    text-align: center;
  }

  .metric-num {
    display: block;
    font-size: 1.05rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .metric-sub {
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .lb-card-footer {
    padding: 0.85rem 1.25rem;
    border-top: 1px solid var(--border-color);
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    background: var(--bg-surface);
  }

  .galera-arbitrator-banner {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-left: 4px solid #f59e0b;
    border-radius: var(--radius-md);
    padding: 1.25rem;
    display: flex;
    align-items: center;
    gap: 1.25rem;
    margin-top: 1rem;
  }

  .banner-icon {
    color: #f59e0b;
  }

  .banner-content {
    flex: 1;
  }

  .banner-title {
    font-weight: 600;
    font-size: 0.95rem;
    margin-bottom: 0.25rem;
  }

  .banner-text {
    font-size: 0.85rem;
    color: var(--text-muted);
    margin: 0;
  }

  .config-code-block {
    background: #1e1e2e;
    color: #cdd6f4;
    padding: 1rem;
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 0.82rem;
    line-height: 1.5;
    max-height: 380px;
    overflow-y: auto;
  }
</style>
