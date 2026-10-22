<script>
  import {
    activeCluster,
    theme,
    switchView,
    openModal
  } from '../lib/stores.js';

  let selectedNodeId = $state(null);

  let primary = $derived(
    $activeCluster ? ($activeCluster.nodes.find(n => n.role === 'primary') || $activeCluster.nodes[0]) : null
  );

  let standbys = $derived(
    $activeCluster && primary ? $activeCluster.nodes.filter(n => n.id !== primary.id) : []
  );

  let isDark = $derived($theme === 'dark');

  // Dynamic canvas width based on standby count to prevent any horizontal crampedness
  let svgWidth = $derived(Math.max(940, (standbys.length + 1) * 310));
  const svgHeight = 440;
  let primaryX = $derived(svgWidth / 2);
  const primaryY = 80;
  const repY = 320;

  let replicaBoxFill = $derived(isDark ? '#162238' : '#ffffff');
  let replicaTextPrimary = $derived(isDark ? '#f8fafc' : '#0f172a');
  let replicaTextMuted = $derived(isDark ? '#94a3b8' : '#64748b');
  let replicaTextMono = $derived(isDark ? '#cbd5e1' : '#475569');
  let lagBadgeBg = $derived(isDark ? '#0d1527' : '#ffffff');

  function getNodeLabel(node) {
    if (!node || !node.id) return 'Node';
    const m = node.id.match(/-node-(\d+)$/i) || node.id.match(/node-(\w+)$/i);
    if (m) {
      return `Node ${m[1]}`;
    }
    return node.id;
  }
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
        <svg viewBox="0 0 {svgWidth} {svgHeight}" width="{svgWidth}" height="{svgHeight}" xmlns="http://www.w3.org/2000/svg">
          <defs>
            <filter id="glowEffect" x="-20%" y="-20%" width="140%" height="140%">
              <feDropShadow dx="0" dy="2" stdDeviation="6" flood-color="#0284c7" flood-opacity="0.35"/>
            </filter>
            <filter id="cardShadow" x="-10%" y="-10%" width="120%" height="120%">
              <feDropShadow dx="0" dy="3" stdDeviation="4" flood-color="#000000" flood-opacity="0.15"/>
            </filter>
            <linearGradient id="primaryGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#0284c7"/>
              <stop offset="100%" stop-color="#0369a1"/>
            </linearGradient>
          </defs>

          <!-- Replicas and Connections -->
          {#each standbys as replica, idx (replica.id)}
            {@const repX = (svgWidth / (standbys.length + 1)) * (idx + 1)}
            {@const isSync = replica.role === 'sync'}
            {@const strokeColor = isSync ? '#059669' : '#0284c7'}
            {@const strokeDash = isSync ? 'none' : '6,6'}
            {@const linkStartBottom = primaryY + 70}
            {@const linkEndTop = repY - 50}
            {@const midY = (linkStartBottom + linkEndTop) / 2}
            {@const midX = (primaryX + repX) / 2}
            {@const isSelected = selectedNodeId === replica.id}

            <!-- Link Path: S-Curve between Primary bottom and Standby top -->
            <path
              d="M {primaryX} {linkStartBottom} C {primaryX} {midY}, {repX} {midY}, {repX} {linkEndTop}"
              fill="none"
              stroke={strokeColor}
              stroke-width={isSync ? '2.8' : '2.2'}
              stroke-dasharray={strokeDash}
              opacity="0.85"
            />

            <!-- Moving stream particle packet -->
            <circle r="4.5" fill={strokeColor} opacity="0.95">
              <animateMotion
                dur={isSync ? '1.5s' : '2.2s'}
                repeatCount="indefinite"
                path="M {primaryX} {linkStartBottom} C {primaryX} {midY}, {repX} {midY}, {repX} {linkEndTop}"
              />
            </circle>

            <!-- Lag Badge (Positioned at midX, midY between primary and replica) -->
            <g transform="translate({midX - 52}, {midY - 14})">
              <rect width="104" height="28" rx="14" fill={lagBadgeBg} stroke={strokeColor} stroke-width="1.6" filter="url(#cardShadow)"/>
              <circle cx="16" cy="14" r="3.5" fill={strokeColor}/>
              <text x="56" y="18" text-anchor="middle" fill={strokeColor} font-size="11" font-weight="700" font-family="JetBrains Mono, monospace">
                {replica.lagMs} ms lag
              </text>
            </g>

            <!-- Standby Node Card (Width: 260px, Height: 120px) -->
            <g
              transform="translate({repX - 130}, {repY - 50})"
              filter="url(#cardShadow)"
              style="cursor: pointer;"
              role="button"
              tabindex="0"
              onclick={() => selectedNodeId = replica.id}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectedNodeId = replica.id; }}
            >
              <!-- Card Background with distinct border -->
              <rect
                width="260"
                height="120"
                rx="10"
                fill={replicaBoxFill}
                stroke={isSelected ? '#38bdf8' : strokeColor}
                stroke-width={isSelected ? '2.5' : '1.8'}
              />

              <!-- Top Row: Role Badge & Health Status (NO text collision) -->
              <rect x="12" y="12" width={isSync ? 105 : 110} height="20" rx="4" fill="{strokeColor}18" stroke="{strokeColor}44" stroke-width="1"/>
              <text x={isSync ? 64 : 67} y="26" text-anchor="middle" fill={strokeColor} font-size="10" font-weight="700" letter-spacing="0.04em">
                {isSync ? 'SYNC STANDBY' : 'ASYNC STANDBY'}
              </text>

              <rect x="182" y="12" width="66" height="20" rx="4" fill="#10b98118" stroke="#10b98144" stroke-width="1"/>
              <circle cx="192" cy="22" r="3" fill="#10b981"/>
              <text x="216" y="26" text-anchor="middle" fill="#10b981" font-size="10" font-weight="700" letter-spacing="0.04em">HEALTHY</text>

              <!-- Line 2: Prominent Node Label (e.g. Node 2) -->
              <text x="14" y="54" fill={replicaTextPrimary} font-size="15" font-weight="700" font-family="Inter, sans-serif">
                {getNodeLabel(replica)}
              </text>

              <!-- Line 3: Full Identifier in clean monospace font -->
              <text x="14" y="73" fill={replicaTextMono} font-size="11" font-family="JetBrains Mono, monospace">
                {replica.id.length > 28 ? replica.id.slice(0, 26) + '…' : replica.id}
              </text>

              <!-- Line 4: Host endpoint & LSN position -->
              <text x="14" y="91" fill={replicaTextMuted} font-size="10.5" font-family="JetBrains Mono, monospace">
                {replica.ip}:{replica.port} • LSN: {replica.lsn}
              </text>

              <!-- Line 5: Stream telemetry state -->
              <text x="14" y="108" fill={strokeColor} font-size="10" font-weight="600" font-family="Inter, sans-serif">
                {isSync ? '• Synchronous replication slot active' : '• Asynchronous WAL stream active'}
              </text>
            </g>
          {/each}

          <!-- Primary Node Card (Center Top, Width: 260px, Height: 120px) -->
          <g
            transform="translate({primaryX - 130}, {primaryY - 50})"
            filter="url(#glowEffect)"
            style="cursor: pointer;"
            role="button"
            tabindex="0"
            onclick={() => selectedNodeId = primary.id}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectedNodeId = primary.id; }}
          >
            <!-- Card Background -->
            <rect
              width="260"
              height="120"
              rx="10"
              fill="url(#primaryGrad)"
              stroke={selectedNodeId === primary.id ? '#ffffff' : '#38bdf8'}
              stroke-width={selectedNodeId === primary.id ? '2.5' : '1.5'}
            />

            <!-- Top Row: Role Badge & Leader Badge (Separated on Left & Right) -->
            <rect x="12" y="12" width="105" height="20" rx="4" fill="#ffffff26"/>
            <text x="64" y="26" text-anchor="middle" fill="#ffffff" font-size="10" font-weight="700" letter-spacing="0.04em">PRIMARY WRITER</text>

            <rect x="188" y="12" width="60" height="20" rx="4" fill="#ffffff33"/>
            <text x="218" y="26" text-anchor="middle" fill="#ffffff" font-size="10" font-weight="800" letter-spacing="0.04em">LEADER</text>

            <!-- Line 2: Prominent Node Label (e.g. Node 1) -->
            <text x="14" y="54" fill="#ffffff" font-size="15" font-weight="700" font-family="Inter, sans-serif">
              {getNodeLabel(primary)}
            </text>

            <!-- Line 3: Full Identifier in clean monospace font -->
            <text x="14" y="73" fill="#e0f2fe" font-size="11" font-family="JetBrains Mono, monospace">
              {primary.id.length > 28 ? primary.id.slice(0, 26) + '…' : primary.id}
            </text>

            <!-- Line 4: Host endpoint & LSN position -->
            <text x="14" y="91" fill="#bae6fd" font-size="10.5" font-family="JetBrains Mono, monospace">
              {primary.ip}:{primary.port} • LSN: {primary.lsn}
            </text>

            <!-- Line 5: Master Write State -->
            <text x="14" y="108" fill="#e0f2fe" font-size="10" font-weight="600" font-family="Inter, sans-serif">
              • Read/Write Master • Epoch #{$activeCluster ? $activeCluster.epoch : 1}
            </text>
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
              <tr class={selectedNodeId === node.id ? 'row-selected' : ''} onclick={() => selectedNodeId = node.id} style="cursor: pointer;">
                <td>
                  <div style="font-weight: 600; color: var(--text-primary);">{node.id}</div>
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
                    <button class="btn btn-secondary btn-sm" onclick={(e) => { e.stopPropagation(); openModal('failover', node); }}>Promote</button>
                  {/if}
                  <button class="btn btn-secondary btn-sm" onclick={(e) => { e.stopPropagation(); openModal('fence', node); }}>Fence</button>
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

  .topology-svg-wrapper {
    overflow-x: auto;
  }

  tr.row-selected {
    background-color: rgba(2, 132, 199, 0.08) !important;
  }

  :global(body.dark-theme) tr.row-selected {
    background-color: rgba(56, 189, 248, 0.12) !important;
  }
</style>
