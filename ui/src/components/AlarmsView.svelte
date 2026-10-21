<script>
  import {
    alerts,
    acknowledgeAlert,
    showToast
  } from '../lib/stores.js';

  function scheduleMaintenance() {
    showToast('Maintenance window opened. Alarms suppressed for 2 hours.', 'info');
  }
</script>

<section class="view-panel active" id="viewAlarms">
  <div class="view-header">
    <div>
      <h1 class="view-title">Active Alarms & Incident Management</h1>
      <p class="view-subtitle">High-severity alerts, replication thresholds, disk pressure warnings, and maintenance suppression.</p>
    </div>
    <div class="view-actions">
      <button class="btn btn-secondary btn-sm" onclick={scheduleMaintenance}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="18" x="3" y="4" rx="2" ry="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/></svg>
        <span>Schedule Maintenance</span>
      </button>
    </div>
  </div>

  <div class="card table-card">
    <div class="card-header">
      <div>
        <h2 class="card-title">Active Alerts</h2>
        <p class="card-desc">Acknowledge alarms to notify teammates or silence during known maintenance.</p>
      </div>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Severity</th>
            <th>Cluster / Node</th>
            <th>Alert Rule</th>
            <th>Message</th>
            <th>Triggered At</th>
            <th>Status</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          {#if $alerts.length === 0}
            <tr>
              <td colspan="7" style="text-align: center; padding: 24px; color: var(--text-muted);">
                ✓ No active alerts. All clusters operating within normal parameters.
              </td>
            </tr>
          {:else}
            {#each $alerts as alt, idx (alt.id)}
              <tr>
                <td><span class="role-pill" style="background: rgba(217, 119, 6, 0.15); color: #d97706; border: 1px solid #d97706;">{alt.severity}</span></td>
                <td style="font-weight: 600;">{alt.target}</td>
                <td>{alt.rule}</td>
                <td style="color: var(--text-secondary);">{alt.message}</td>
                <td style="color: var(--text-muted);">{alt.triggeredAt}</td>
                <td><span class="status-indicator text-amber">{alt.status}</span></td>
                <td>
                  <button class="btn btn-secondary btn-sm" onclick={() => acknowledgeAlert(idx)}>Acknowledge</button>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</section>
