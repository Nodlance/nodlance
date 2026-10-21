<script>
  import {
    hosts,
    openModal,
    showToast
  } from '../lib/stores.js';

  function diagnoseHost(name) {
    showToast(`Diagnostics started for host ${name}. Connectivity OK.`, 'info');
  }
</script>

<section class="view-panel active" id="viewHosts">
  <div class="view-header">
    <div>
      <h1 class="view-title">Host Inventory & Agent Management</h1>
      <p class="view-subtitle">Physical servers, virtual machines, container hosts, and enrolled Nodlance agent daemons.</p>
    </div>
    <div class="view-actions">
      <button class="btn btn-primary btn-sm" onclick={() => openModal('enroll')}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><line x1="19" x2="19" y1="8" y2="14"/><line x1="22" x2="16" y1="11" y2="11"/></svg>
        <span>Enroll New Host</span>
      </button>
    </div>
  </div>

  <div class="card table-card">
    <div class="card-header">
      <div>
        <h2 class="card-title">Registered Host Infrastructure</h2>
        <p class="card-desc">Live agent heartbeats, CPU/memory telemetry, and SSH/systemd enrollment tokens.</p>
      </div>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Host Name</th>
            <th>IP Address</th>
            <th>OS Distribution</th>
            <th>Agent Status</th>
            <th>Heartbeat</th>
            <th>CPU Usage</th>
            <th>RAM Usage</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each $hosts as h (h.name)}
            <tr>
              <td style="font-weight: 600;">{h.name}</td>
              <td class="font-mono">{h.ip}</td>
              <td>{h.os}</td>
              <td><span class="status-indicator text-emerald"><span class="pulse-indicator"></span> {h.agentStatus}</span></td>
              <td class="font-mono">&lt; {h.heartbeatSec}s ago</td>
              <td>
                <div style="display: flex; align-items: center; gap: 6px;">
                  <div style="flex: 1; height: 6px; background: var(--border-subtle); border-radius: 3px; overflow: hidden;">
                    <div style="width: {h.cpu}%; height: 100%; background: var(--brand-accent);"></div>
                  </div>
                  <span class="font-mono" style="font-size: 11px;">{h.cpu}%</span>
                </div>
              </td>
              <td>
                <div style="display: flex; align-items: center; gap: 6px;">
                  <div style="flex: 1; height: 6px; background: var(--border-subtle); border-radius: 3px; overflow: hidden;">
                    <div style="width: {h.ram}%; height: 100%; background: #059669;"></div>
                  </div>
                  <span class="font-mono" style="font-size: 11px;">{h.ram}%</span>
                </div>
              </td>
              <td>
                <button class="btn btn-secondary btn-sm" onclick={() => diagnoseHost(h.name)}>Diagnose</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</section>
