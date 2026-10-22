<script>
  import {
    activeCluster,
    theme,
    switchView,
    openModal
  } from '../lib/stores.js';

  let primary = $derived(
    $activeCluster ? ($activeCluster.nodes.find(n => n.role === 'primary') || $activeCluster.nodes[0]) : null
  );

  let standbys = $derived(
    $activeCluster && primary ? $activeCluster.nodes.filter(n => n.id !== primary.id) : []
  );

  const svgWidth = 840;
  const svgHeight = 360;
  const primaryX = 420;
  const primaryY = 70;

  let isDark = $derived($theme === 'dark');
  let replicaBoxFill = $derived(isDark ? '#162238' : '#ffffff');
  let replicaTextPrimary = $derived(isDark ? '#f8fafc' : '#0f172a');
  let replicaTextMuted = $derived(isDark ? '#cbd5e1' : '#64748b');
  let lagBadgeBg = $derived(isDark ? '#0d1527' : '#ffffff');
  let lagBadgeBorder = $derived(isDark ? '#243552' : '#e2e8f0');
</script>

<section class="view-panel active" id="viewTopology">
  <div class="view-header">
    <div>
      <div class="cluster-breadcrumbs">
        <button class="crumb-parent btn-link" onclick={() => switchView('fleet')}>Clusters</button>
        <span class="crumb-sep">/</span>
        <span class="crumb-current">{$activeCluster ? $activeCluster.name : ''}</span>
      </div>
      <h1 class="view-title">Replication Topology & Node Graph</h1>
      <p class="view-subtitle">Live interactive replication hierarchy, WAL/GTID positions, latency indicators, and node-level control actions.</p>
    </div>
    <div class="view-actions">
      <button class="btn btn-secondary btn-sm" onclick={() => switchView('ha-recovery')} title="Open HA Failover Studio for this cluster">
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m11 17 5-5-5-5"/><path d="M18 12H4"/></svg>
        <span>HA Failover Studio</span>
      </button>
      <button class="btn btn-secondary btn-sm" onclick={() => openModal('backup')}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/></svg>
        <span>Take Backup</span>
      </button>
    </div>
  </div>

  <!-- Interactive SVG Topology Graph Container -->
  <div class="topology-canvas-card">
    <div class="topology-toolbar">
      <div class="topo-legend">
        <span class="legend-item"><span class="legend-node primary"></span> Primary / Leader</span>
        <span class="legend-item"><span class="legend-node sync"></span> Sync Standby</span>
        <span class="legend-item"><span class="legend-node async"></span> Async Standby</span>
        <span class="legend-item"><span class="legend-node fenced"></span> Fenced / Isolated</span>
        <span class="legend-item"><span class="legend-node vip"></span> Virtual IP / Proxy</span>
      </div>
      <div class="topo-controls">
        <span class="refresh-badge"><span class="pulse-indicator"></span> Live Replication Stream</span>
      </div>
    </div>

    <div class="topology-svg-wrapper">
      {#if primary}
        <svg viewBox="0 0 {svgWidth} {svgHeight}" width="100%" height="360" xmlns="http://www.w3.org/2000/svg">
          <defs>
            <filter id="glowEffect" x="-20%" y="-20%" width="140%" height="140%">
              <feDropShadow dx="0" dy="0" stdDeviation="6" flood-color="#0284c7" flood-opacity="0.3"/>
            </filter>
            <linearGradient id="primaryGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#0284c7"/>
              <stop offset="100%" stop-color="#38bdf8"/>
            </linearGradient>
          </defs>

          <!-- Replicas and Connections -->
          {#each standbys as replica, idx (replica.id)}
            {@const repX = (svgWidth / (standbys.length + 1)) * (idx + 1)}
            {@const repY = 260}
            {@const isSync = replica.role === 'sync'}
            {@const strokeColor = isSync ? '#059669' : '#0284c7'}
            {@const strokeDash = isSync ? 'none' : '5,5'}

            <!-- Link -->
            <path d="M {primaryX} {primaryY + 36} C {primaryX} {(primaryY + repY) / 2}, {repX} {(primaryY + repY) / 2}, {repX} {repY - 36}"
                  fill="none" stroke={strokeColor} stroke-width="2.5" stroke-dasharray={strokeDash} opacity="0.85"/>

            <!-- Moving packet -->
            <circle r="4" fill={strokeColor}>
              <animateMotion dur={isSync ? '1.4s' : '2.2s'} repeatCount="indefinite"
                path="M {primaryX} {primaryY + 36} C {primaryX} {(primaryY + repY) / 2}, {repX} {(primaryY + repY) / 2}, {repX} {repY - 36}"/>
            </circle>

            <!-- Lag Badge -->
            <g transform="translate({(primaryX + repX) / 2 - 35}, {(primaryY + repY) / 2 - 12})">
              <rect width="70" height="22" rx="11" fill={lagBadgeBg} stroke={lagBadgeBorder} stroke-width="1.2"/>
              <text x="35" y="15" text-anchor="middle" fill={strokeColor} font-size="10" font-weight="700" font-family="JetBrains Mono">
                {replica.lagMs}ms lag
              </text>
            </g>

            <!-- Node Card -->
            <g transform="translate({repX - 90}, {repY - 32})" style="cursor: pointer;">
              <rect width="180" height="74" rx="8" fill={replicaBoxFill} stroke={strokeColor} stroke-width="1.5"/>
              <text x="14" y="24" fill={replicaTextPrimary} font-size="12" font-weight="700" font-family="Inter">{replica.id}</text>
              <text x="14" y="42" fill={replicaTextMuted} font-size="10" font-family="Inter">{replica.ip}:{replica.port}</text>
              <text x="14" y="58" fill={strokeColor} font-size="10" font-weight="600" font-family="JetBrains Mono">{replica.role.toUpperCase()} STANDBY</text>
              <rect x="125" y="12" width="42" height="18" rx="4" fill="{strokeColor}22"/>
              <text x="146" y="25" text-anchor="middle" fill={strokeColor} font-size="9" font-weight="700">OK</text>
            </g>
          {/each}

          <!-- Primary Node Card (Center Top) -->
          <g transform="translate({primaryX - 110}, {primaryY - 36})" filter="url(#glowEffect)" style="cursor: pointer;">
            <rect width="220" height="84" rx="10" fill="url(#primaryGrad)"/>
            <text x="16" y="28" fill="#ffffff" font-size="14" font-weight="700" font-family="Inter">{primary.id}</text>
            <text x="16" y="48" fill="#e0f2fe" font-size="11" font-family="Inter">{primary.host}</text>
            <text x="16" y="68" fill="#ffffff" font-size="11" font-weight="800" font-family="JetBrains Mono">PRIMARY (WRITER)</text>
            <rect x="155" y="14" width="50" height="20" rx="4" fill="#ffffff33"/>
            <text x="180" y="28" text-anchor="middle" fill="#ffffff" font-size="10" font-weight="800">LEADER</text>
          </g>
        </svg>
      {/if}
    </div>
  </div>

  <!-- Node List Table with Deep Telemetry -->
  <div class="card table-card">
    <div class="card-header">
      <div>
        <h2 class="card-title">Cluster Node Instances</h2>
        <span class="table-subtitle">Select any node row to view comprehensive diagnostics, buffer cache stats, and fencing triggers.</span>
      </div>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Node ID / Host</th>
            <th>Role</th>
            <th>State</th>
            <th>IP / Port</th>
            <th>Replication Lag</th>
            <th>WAL / GTID LSN</th>
            <th>CPU / Mem</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#if $activeCluster && $activeCluster.nodes}
            {#each $activeCluster.nodes as node (node.id)}
              {@const isPrimary = node.role === 'primary'}
              <tr>
                <td>
                  <div style="font-weight: 600;">{node.id}</div>
                  <div style="font-size: 11px; color: var(--text-muted);">{node.host}</div>
                </td>
                <td>
                  <span class="role-pill {node.role}">{node.role.toUpperCase()}</span>
                </td>
                <td>
                  <span class="status-indicator text-emerald">
                    <span class="pulse-indicator"></span> {node.state.toUpperCase()}
                  </span>
                </td>
                <td class="font-mono">{node.ip}:{node.port}</td>
                <td>
                  <span style="font-weight: 600; color: {node.lagMs === 0 ? 'var(--color-success)' : 'var(--brand-accent)'};">
                    {node.lagMs} ms
                  </span>
                  <span style="font-size: 11px; color: var(--text-muted);">({node.lagBytes} B)</span>
                </td>
                <td class="font-mono" style="font-size: 11px;">{node.lsn}</td>
                <td>{node.cpu} / {node.mem}</td>
                <td>
                  {#if isPrimary}
                    <button class="btn btn-secondary btn-sm" disabled title="Leader node">Leader</button>
                  {:else}
                    <button class="btn btn-secondary btn-sm" onclick={() => openModal('failover', node)}>Promote</button>
                  {/if}
                  <button class="btn btn-secondary btn-sm" onclick={() => openModal('fence', node)}>Fence</button>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</section>

<style>
  .btn-link {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: inherit;
    font-family: inherit;
    cursor: pointer;
    padding: 0;
  }
  .btn-link:hover {
    color: var(--brand-accent);
  }
</style>
