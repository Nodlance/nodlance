<script>
  import { loadBalancers, reloadLoadBalancer, openModal, showToast } from '../lib/stores.js';

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

<section class="view-panel active" id="viewLoadBalancers">
  <!-- View Header -->
  <div class="view-header">
    <div>
      <h1 class="view-title">Load Balancers & Connection Routers</h1>
      <p class="view-subtitle">Severalnines ClusterControl Parity • ProxySQL, PgBouncer, HAProxy, and Keepalived VRRP VIP</p>
    </div>
    <div class="view-actions">
      <button class="btn btn-primary btn-sm" onclick={() => openModal('deployLoadBalancer')}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        <span>Deploy Router / VIP</span>
      </button>
    </div>
  </div>

  <!-- Summary KPI Cards -->
  <div class="metrics-grid">
    <div class="metric-card">
      <div class="metric-label">ACTIVE ROUTERS & POOLERS</div>
      <div class="metric-val">{$loadBalancers.length} <span class="metric-unit">Active</span></div>
      <div class="metric-trend positive">ProxySQL, PgBouncer & Keepalived</div>
    </div>
    <div class="metric-card">
      <div class="metric-label">TOTAL ROUTED THROUGHPUT</div>
      <div class="metric-val">30,550 <span class="metric-unit">QPS</span></div>
      <div class="metric-trend positive">Sub-millisecond query routing</div>
    </div>
    <div class="metric-card">
      <div class="metric-label">VIRTUAL IP (VRRP)</div>
      <div class="metric-val font-mono">192.168.10.100</div>
      <div class="metric-trend positive">Master: Node 01 (VRID 51)</div>
    </div>
    <div class="metric-card">
      <div class="metric-label">FAILOVER PROTECTION</div>
      <div class="metric-val text-emerald">Automated</div>
      <div class="metric-trend positive">Synced with STONITH Fencing</div>
    </div>
  </div>

  <!-- Filter tabs -->
  <div class="lb-filter-bar">
    <div class="filter-group">
      <button class="filter-btn {selectedType === 'all' ? 'active' : ''}" onclick={() => selectedType = 'all'}>
        All Instances ({$loadBalancers.length})
      </button>
      <button class="filter-btn {selectedType === 'proxysql' ? 'active' : ''}" onclick={() => selectedType = 'proxysql'}>
        ProxySQL ({$loadBalancers.filter(l => l.type === 'proxysql').length})
      </button>
      <button class="filter-btn {selectedType === 'pgbouncer' ? 'active' : ''}" onclick={() => selectedType = 'pgbouncer'}>
        PgBouncer ({$loadBalancers.filter(l => l.type === 'pgbouncer').length})
      </button>
      <button class="filter-btn {selectedType === 'keepalived' ? 'active' : ''}" onclick={() => selectedType = 'keepalived'}>
        Keepalived VIP ({$loadBalancers.filter(l => l.type === 'keepalived').length})
      </button>
    </div>
    <div class="lb-filter-info">
      Showing <strong>{filteredLbs.length}</strong> active routing instance{filteredLbs.length === 1 ? '' : 's'}
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
            <span class="lb-prop-value font-mono host-pill">{lb.host} ({lb.ip})</span>
          </div>
          <div class="lb-prop-row">
            <span class="lb-prop-label">Routing Mode:</span>
            <span class="lb-prop-value routing-mode-pill">{lb.mode}</span>
          </div>
          <div class="lb-prop-row">
            <span class="lb-prop-label">Ports:</span>
            <span class="lb-prop-value font-mono ports-pill">
              Listen: <span class="port-accent">{lb.listenPort}</span>
              {#if lb.adminPort} • Admin: <span class="port-accent">{lb.adminPort}</span>{/if}
            </span>
          </div>
          <div class="lb-prop-row backends-row">
            <span class="lb-prop-label">Backends:</span>
            <div class="backends-tags">
              <span class="backend-tag primary">{lb.writers}</span>
              <span class="backend-tag replica">{lb.readers}</span>
            </div>
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
              <span class="metric-sub">Multiplex</span>
            </div>
          </div>
        </div>

        <div class="lb-card-footer">
          <button class="btn btn-secondary btn-sm" onclick={() => viewConfig(lb)}>
            <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
            <span>View Config</span>
          </button>
          <button class="btn btn-primary btn-sm" onclick={() => handleReload(lb)}>
            <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 0 1 15-6.7L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-15 6.7L3 16"/><path d="M3 21v-5h5"/></svg>
            <span>Sync Rules</span>
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
      <button class="btn btn-secondary btn-sm" onclick={() => {
        showToast('Generated garbd arbitrator unit for core-mysql-ecommerce (odd-node quorum secured)', 'success');
      }}>
        Generate Garbd Unit
      </button>
    </div>
  </div>
</section>

<!-- Modal for Viewing Generated Configuration -->
{#if showConfigModal && selectedLbModal}
  <div
    class="modal-backdrop open"
    onclick={(e) => { if (e.target === e.currentTarget) showConfigModal = false; }}
    onkeydown={(e) => { if (e.key === 'Escape') showConfigModal = false; }}
    role="dialog"
    tabindex="-1"
  >
    <div class="modal-window modal-lg">
      <div class="modal-header">
        <h3 class="modal-title">Live Generated Configuration: {selectedLbModal.name}</h3>
        <button class="modal-close-btn" onclick={() => showConfigModal = false}>&times;</button>
      </div>
      <div class="modal-body">
        <p class="step-desc">
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
  .lb-filter-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-5);
    flex-wrap: wrap;
    gap: var(--space-3);
  }

  .lb-filter-info {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .lb-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
    gap: var(--space-4);
    margin-bottom: var(--space-6);
  }

  .lb-card {
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: var(--shadow-sm);
    transition: transform var(--transition-fast), border-color var(--transition-fast), box-shadow var(--transition-fast);
  }

  .lb-card:hover {
    transform: translateY(-2px);
    border-color: var(--brand-accent);
    box-shadow: var(--shadow-md);
  }

  .lb-card-header {
    padding: 14px 18px;
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--bg-surface);
  }

  .lb-brand {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .lb-badge {
    font-size: 11px;
    font-weight: 700;
    padding: 2px 7px;
    border-radius: 4px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .lb-badge-proxysql {
    background: rgba(2, 132, 199, 0.12);
    color: #0284c7;
    border: 1px solid rgba(2, 132, 199, 0.3);
  }

  .lb-badge-pgbouncer {
    background: rgba(124, 58, 237, 0.12);
    color: #7c3aed;
    border: 1px solid rgba(124, 58, 237, 0.3);
  }

  .lb-badge-keepalived {
    background: rgba(217, 119, 6, 0.12);
    color: #d97706;
    border: 1px solid rgba(217, 119, 6, 0.3);
  }

  .lb-badge-haproxy {
    background: rgba(5, 150, 105, 0.12);
    color: #059669;
    border: 1px solid rgba(5, 150, 105, 0.3);
  }

  :global(body.dark-theme) .lb-badge-proxysql { color: #38bdf8; }
  :global(body.dark-theme) .lb-badge-pgbouncer { color: #c084fc; }
  :global(body.dark-theme) .lb-badge-keepalived { color: #fbbf24; }
  :global(body.dark-theme) .lb-badge-haproxy { color: #34d399; }

  .lb-name {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 14px;
  }

  .lb-card-body {
    padding: 16px 18px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex: 1;
  }

  .lb-prop-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
  }

  .lb-prop-row.backends-row {
    align-items: flex-start;
  }

  .lb-prop-label {
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
  }

  .lb-prop-value {
    color: var(--text-primary);
    font-size: 12.5px;
  }

  .fw-600 {
    font-weight: 600;
  }

  .host-pill, .ports-pill {
    background: var(--bg-surface);
    padding: 2px 7px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }

  .routing-mode-pill {
    background: rgba(2, 132, 199, 0.08);
    color: var(--brand-accent);
    padding: 2px 7px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    font-size: 12px;
    border: 1px solid rgba(2, 132, 199, 0.2);
  }

  .port-accent {
    font-weight: 700;
    color: var(--brand-accent);
  }

  .backends-tags {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: flex-end;
  }

  .backend-tag {
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
  }

  .backend-tag.primary {
    border-color: rgba(2, 132, 199, 0.3);
    color: var(--brand-accent);
  }

  .lb-metrics-strip {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    margin-top: 10px;
    padding: 10px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    text-align: center;
  }

  .metric-item {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .metric-num {
    font-size: 15px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .metric-sub {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-top: 2px;
  }

  .lb-card-footer {
    padding: 12px 18px;
    border-top: 1px solid var(--border-subtle);
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    background: var(--bg-surface);
  }

  .galera-arbitrator-banner {
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    border-left: 4px solid var(--color-warning);
    border-radius: var(--radius-md);
    padding: 16px 20px;
    display: flex;
    align-items: center;
    gap: 16px;
    margin-top: var(--space-5);
    box-shadow: var(--shadow-sm);
  }

  .banner-icon {
    color: var(--color-warning);
    display: flex;
    align-items: center;
  }

  .banner-content {
    flex: 1;
  }

  .banner-title {
    font-weight: 700;
    font-size: 14px;
    margin-bottom: 2px;
    color: var(--text-primary);
  }

  .banner-text {
    font-size: 12.5px;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.45;
  }

  .config-code-block {
    background: #0d1527;
    color: #e2e8f0;
    padding: 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.55;
    max-height: 400px;
    overflow-y: auto;
  }
</style>
