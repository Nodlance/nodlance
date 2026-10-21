<script>
  import {
    clusters,
    engineFilter,
    activeClusterId,
    switchView,
    showToast
  } from '../lib/stores.js';

  let filteredClusters = $derived(
    $clusters.filter(c => {
      if ($engineFilter === 'all') return true;
      return c.engine === $engineFilter;
    })
  );

  let totalThroughput = $derived(
    $clusters.reduce((acc, c) => acc + (c.qps || 0), 0)
  );

  let totalStorage = $derived(
    ($clusters.reduce((acc, c) => acc + (c.storageGb || 0), 0) / 1000).toFixed(2)
  );

  function viewTopologyFor(clusterId) {
    activeClusterId.set(clusterId);
    switchView('topology');
  }

  function refreshFleet() {
    showToast('Refreshing cluster telemetry from Nodlance Controller...', 'info');
  }
</script>

<section class="view-panel active" id="viewFleet">
  <div class="view-header">
    <div>
      <h1 class="view-title">Database Fleet Overview</h1>
      <p class="view-subtitle">Real-time status, health metrics, and replication telemetry across all managed database clusters.</p>
    </div>
    <div class="view-actions">
      <div class="filter-group">
        <button class="filter-btn {$engineFilter === 'all' ? 'active' : ''}" onclick={() => engineFilter.set('all')}>All Engines</button>
        <button class="filter-btn {$engineFilter === 'postgres' ? 'active' : ''}" onclick={() => engineFilter.set('postgres')}>PostgreSQL</button>
        <button class="filter-btn {$engineFilter === 'mysql' ? 'active' : ''}" onclick={() => engineFilter.set('mysql')}>MySQL</button>
        <button class="filter-btn {$engineFilter === 'mariadb' ? 'active' : ''}" onclick={() => engineFilter.set('mariadb')}>MariaDB</button>
      </div>
      <button class="btn btn-secondary btn-sm" onclick={refreshFleet}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/></svg>
        <span>Refresh</span>
      </button>
    </div>
  </div>

  <!-- Global Telemetry Cards -->
  <div class="metrics-grid">
    <div class="metric-card">
      <div class="metric-label">TOTAL CLUSTER THROUGHPUT</div>
      <div class="metric-val">{totalThroughput.toLocaleString()} <span class="metric-unit">QPS</span></div>
      <div class="metric-trend positive">↑ 8.4% from last hour</div>
    </div>
    <div class="metric-card">
      <div class="metric-label">AVG REPLICATION LAG</div>
      <div class="metric-val">14 <span class="metric-unit">ms</span></div>
      <div class="metric-trend neutral">Within SLA (&lt; 100ms)</div>
    </div>
    <div class="metric-card">
      <div class="metric-label">AGGREGATE STORAGE</div>
      <div class="metric-val">{totalStorage} <span class="metric-unit">TB</span></div>
      <div class="metric-trend neutral">46% Capacity Used</div>
    </div>
    <div class="metric-card">
      <div class="metric-label">HA QUORUM STABILITY</div>
      <div class="metric-val text-emerald">100%</div>
      <div class="metric-trend positive">All epochs synchronized</div>
    </div>
  </div>

  <!-- Clusters Grid -->
  <div class="cluster-cards-grid">
    {#each filteredClusters as c (c.id)}
      <div
        class="cluster-card"
        onclick={() => viewTopologyFor(c.id)}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') viewTopologyFor(c.id); }}
        role="button"
        tabindex="0"
      >
        <div class="cluster-card-head">
          <div>
            <span class="cluster-engine-badge {c.engine === 'postgres' ? 'pg' : c.engine === 'mysql' ? 'my' : 'ma'}">
              {c.engineName}
            </span>
            <h3 class="cluster-name">{c.name}</h3>
            <span class="cluster-id">{c.id}</span>
          </div>
          <span class="cluster-status-pill {c.status}">{c.status.toUpperCase()}</span>
        </div>

        <div class="cluster-stats-row">
          <div class="cluster-mini-stat">
            <span class="mini-stat-label">Throughput</span>
            <span class="mini-stat-val">{(c.qps || 0).toLocaleString()} QPS</span>
          </div>
          <div class="cluster-mini-stat">
            <span class="mini-stat-label">Replication Lag</span>
            <span class="mini-stat-val text-emerald">{c.avgLagMs || 0} ms</span>
          </div>
          <div class="cluster-mini-stat">
            <span class="mini-stat-label">Nodes</span>
            <span class="mini-stat-val">{c.nodes ? c.nodes.length : 0} Active</span>
          </div>
        </div>

        <div class="cluster-card-foot">
          <div class="cluster-provider-tag">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="14" x="2" y="3" rx="2"/></svg>
            <span>{c.provider}</span>
          </div>
          <button class="btn btn-secondary btn-sm" onclick={(e) => { e.stopPropagation(); viewTopologyFor(c.id); }}>
            View Topology →
          </button>
        </div>
      </div>
    {/each}
  </div>
</section>
