<script>
  import {
    clusters,
    activeClusterId,
    theme,
    toggleTheme,
    isDemoMode,
    alerts,
    openModal,
    showToast
  } from '../lib/stores.js';

  let totalNodes = $derived(
    $clusters.reduce((acc, c) => acc + (c.nodes ? c.nodes.length : 0), 0)
  );

  function handleClusterChange(e) {
    activeClusterId.set(e.target.value);
    const chosen = $clusters.find(c => c.id === e.target.value);
    showToast(`Active cluster switched to: ${chosen ? chosen.name : ''}`, 'info');
  }

  function handleToggleMode() {
    isDemoMode.update(v => !v);
    if ($isDemoMode) {
      showToast('Switched to Demo Telemetry Mode', 'info');
    } else {
      showToast('Connected to Live API on :8080', 'success');
    }
  }
</script>

<header class="top-nav" id="topNav">
  <div class="nav-left">
    <div class="brand-container">
      <img src="/assets/logo.svg" alt="Nodlance Logo" class="brand-logo" />
      <span class="brand-badge">ClusterControl</span>
    </div>

    <div class="cluster-quick-select-wrap">
      <label for="clusterSelect" class="sr-only">Active Cluster</label>
      <select id="clusterSelect" class="cluster-select" value={$activeClusterId} onchange={handleClusterChange} title="Switch active database cluster">
        {#each $clusters as c}
          <option value={c.id}>{c.name} ({c.engine.toUpperCase()})</option>
        {/each}
      </select>
    </div>
  </div>

  <!-- Cluster Fleet Summary Pills (ClusterControl signature) -->
  <div class="fleet-pill-bar">
    <div class="stat-pill" title="Total active managed database clusters">
      <span class="pill-dot emerald"></span>
      <span class="pill-label">Clusters:</span>
      <span class="pill-val">{$clusters.length}</span>
    </div>
    <div class="stat-pill" title="Total managed database nodes">
      <span class="pill-dot sky"></span>
      <span class="pill-label">Nodes:</span>
      <span class="pill-val">{totalNodes}</span>
    </div>
    <div class="stat-pill" title="Active alerts requiring attention">
      <span class="pill-dot amber"></span>
      <span class="pill-label">Alerts:</span>
      <span class="pill-val">{$alerts.length}</span>
    </div>
    <div class="stat-pill" title="Global HA Quorum status across clusters">
      <span class="pill-dot emerald"></span>
      <span class="pill-label">Quorum:</span>
      <span class="pill-val">Healthy</span>
    </div>
  </div>

  <div class="nav-right">
    <!-- Theme Switcher (Light Default, Dark Optional) -->
    <button class="theme-toggle-btn" onclick={toggleTheme} title="Switch Light/Dark mode">
      {#if $theme === 'light'}
        <svg class="theme-icon sun-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>
        </svg>
        <span class="theme-label">Light</span>
      {:else}
        <svg class="theme-icon moon-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>
        </svg>
        <span class="theme-label">Dark</span>
      {/if}
    </button>

    <!-- Data Mode Switcher (Live API vs Demo Fleet) -->
    <button class="mode-toggle-btn" onclick={handleToggleMode} title="Toggle Demo / Live mode">
      <span class="mode-dot live"></span>
      <span class="mode-text">{$isDemoMode ? 'Demo Fleet' : 'Live API'}</span>
    </button>

    <!-- Primary Action Buttons -->
    <button class="btn btn-secondary btn-sm" onclick={() => openModal('scan')} title="Scan infrastructure for unmanaged database instances">
      <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
      <span>Discover</span>
    </button>
    <button class="btn btn-primary btn-sm" onclick={() => openModal('deployWizard')} title="Deploy a new high-availability database cluster">
      <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
      <span>Deploy Cluster</span>
    </button>

    <!-- User Profile / Session -->
    <div class="user-menu-wrap">
      <div class="user-avatar" tabindex="0" role="button" aria-label="User account settings">
        <span class="avatar-text">AD</span>
        <span class="online-indicator"></span>
      </div>
    </div>
  </div>
</header>
