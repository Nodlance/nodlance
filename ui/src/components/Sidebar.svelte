<script>
  import { activeView, switchView, alerts } from '../lib/stores.js';
</script>

<aside class="sidebar" id="sidebar">
  <nav class="sidebar-nav" aria-label="Primary Navigation">
    <div class="nav-section-title">CLUSTER OPERATIONS</div>
    <button class="nav-item {$activeView === 'fleet' ? 'active' : ''}" onclick={() => switchView('fleet')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="7" height="7" x="3" y="3" rx="1"/><rect width="7" height="7" x="14" y="3" rx="1"/><rect width="7" height="7" x="14" y="14" rx="1"/><rect width="7" height="7" x="3" y="14" rx="1"/></svg>
      <span>Fleet Overview</span>
    </button>
    <button class="nav-item {$activeView === 'topology' ? 'active' : ''}" onclick={() => switchView('topology')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="5" r="3"/><circle cx="6" cy="19" r="3"/><circle cx="18" cy="19" r="3"/><path d="M12 8v4"/><path d="m6.8 16.2 3.4-3.4"/><path d="m17.2 16.2-3.4-3.4"/></svg>
      <span>Topology & Nodes</span>
    </button>
    <button class="nav-item {$activeView === 'load-balancers' ? 'active' : ''}" onclick={() => switchView('load-balancers')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><circle cx="6" cy="6" r="3"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="18" r="3"/><path d="m9 9 6 6"/><path d="m15 9-6 6"/></svg>
      <span>Load Balancers</span>
      <span class="nav-badge">Proxy</span>
    </button>
    <button class="nav-item {$activeView === 'ha-recovery' ? 'active' : ''}" onclick={() => switchView('ha-recovery')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10"/><path d="m9 12 2 2 4-4"/></svg>
      <span>HA & Recovery</span>
      <span class="nav-badge">Quorum</span>
    </button>
    <button class="nav-item {$activeView === 'backups' ? 'active' : ''}" onclick={() => switchView('backups')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/></svg>
      <span>Backups & Drills</span>
    </button>
    <button class="nav-item {$activeView === 'advisors' ? 'active' : ''}" onclick={() => switchView('advisors')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v4"/><path d="m4.93 4.93 2.83 2.83"/><path d="M2 12h4"/><path d="m4.93 19.07 2.83-2.83"/><path d="M12 22v-4"/><path d="m19.07 19.07-2.83-2.83"/><path d="M22 12h-4"/><path d="m19.07 4.93-2.83 2.83"/></svg>
      <span>DBA Advisors</span>
      <span class="nav-badge" style="background: rgba(168, 85, 247, 0.2); color: #c084fc;">Rhai</span>
    </button>
    <button class="nav-item {$activeView === 'config-pki' ? 'active' : ''}" onclick={() => switchView('config-pki')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
      <span>Config & PKI CA</span>
      <span class="nav-badge" style="background: rgba(16, 185, 129, 0.2); color: #34d399;">mTLS</span>
    </button>

    <div class="nav-section-title">PROVISIONING & INFRA</div>
    <button class="nav-item {$activeView === 'provisioning' ? 'active' : ''}" onclick={() => switchView('provisioning')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m16 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"/><path d="m2 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"/><path d="M7 21h10"/><path d="M12 3v18"/><path d="M3 7h2c2 0 5-1 7-2 2 1 5 2 7 2h2"/></svg>
      <span>Guarded Provision</span>
    </button>
    <button class="nav-item {$activeView === 'discovery' ? 'active' : ''}" onclick={() => switchView('discovery')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 12h20"/><path d="M20 12v8H4v-8"/><path d="m4 4 16 0"/><circle cx="12" cy="12" r="3"/></svg>
      <span>Discovery Scanner</span>
    </button>
    <button class="nav-item {$activeView === 'hosts' ? 'active' : ''}" onclick={() => switchView('hosts')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="8" x="2" y="2" rx="2" ry="2"/><rect width="20" height="8" x="2" y="14" rx="2" ry="2"/><line x1="6" x2="6.01" y1="6" y2="6"/><line x1="6" x2="6.01" y1="18" y2="18"/></svg>
      <span>Host Inventory</span>
    </button>

    <div class="nav-section-title">OBSERVABILITY & AUDIT</div>
    <button class="nav-item {$activeView === 'alarms' ? 'active' : ''}" onclick={() => switchView('alarms')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/></svg>
      <span>Active Alarms</span>
      {#if $alerts.length > 0}
        <span class="nav-badge alert-count">{$alerts.length}</span>
      {/if}
    </button>
    <button class="nav-item {$activeView === 'audit' ? 'active' : ''}" onclick={() => switchView('audit')}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" x2="8" y1="13" y2="13"/><line x1="16" x2="8" y1="17" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
      <span>Audit Log</span>
    </button>
  </nav>

  <div class="sidebar-footer">
    <div class="system-status-card">
      <div class="sys-status-head">
        <span class="pulse-indicator"></span>
        <span class="sys-title">Nodlance Controller</span>
      </div>
      <div class="sys-meta">v0.1.0-alpha • Svelte 5 & Axum</div>
      <div class="sys-meta">Uptime: 4d 18h 22m</div>
    </div>
  </div>
</aside>
