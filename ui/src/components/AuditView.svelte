<script>
  import { audit } from '../lib/stores.js';
</script>

<section class="view-panel active" id="viewAudit">
  <div class="view-header">
    <div>
      <h1 class="view-title">Tamper-Evident Audit Trail</h1>
      <p class="view-subtitle">Append-only security log recording every failover, configuration change, user login, and fencing event.</p>
    </div>
  </div>

  <div class="card table-card">
    <div class="card-header">
      <div>
        <h2 class="card-title">Audit Log Entries</h2>
        <p class="card-desc">Cryptographically attributable operator and automated agent activities.</p>
      </div>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Event ID</th>
            <th>Timestamp</th>
            <th>Actor</th>
            <th>Action</th>
            <th>Resource Target</th>
            <th>Status</th>
            <th>Client IP</th>
          </tr>
        </thead>
        <tbody>
          {#each $audit as a (a.id)}
            <tr>
              <td class="font-mono">{a.id}</td>
              <td class="font-mono" style="color: var(--text-muted);">{a.time}</td>
              <td style="font-weight: 600;">{a.actor}</td>
              <td><span class="role-pill primary">{a.action}</span></td>
              <td>{a.target}</td>
              <td><span class="status-indicator text-emerald">{a.status}</span></td>
              <td class="font-mono">{a.ip}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</section>
