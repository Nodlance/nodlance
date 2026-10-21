<script>
  import {
    discovery,
    openModal,
    importDiscoveredWorkload
  } from '../lib/stores.js';
</script>

<section class="view-panel active" id="viewDiscovery">
  <div class="view-header">
    <div>
      <h1 class="view-title">Workload Discovery Scanner</h1>
      <p class="view-subtitle">Scan subnets, local Docker daemon, or Kubernetes pods for existing unmanaged PostgreSQL, MySQL, and MariaDB instances.</p>
    </div>
    <div class="view-actions">
      <button class="btn btn-primary btn-sm" onclick={() => openModal('scan')}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
        <span>Run Discovery Scan</span>
      </button>
    </div>
  </div>

  <div class="card table-card">
    <div class="card-header">
      <div>
        <h2 class="card-title">Discovered Unmanaged Workloads</h2>
        <p class="card-desc">Import existing unmanaged database workloads directly into Nodlance ClusterControl with zero downtime.</p>
      </div>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Workload ID</th>
            <th>Host / IP</th>
            <th>Port</th>
            <th>Detected Engine</th>
            <th>Version</th>
            <th>Topology Role</th>
            <th>Discovered At</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          {#each $discovery as d (d.id)}
            <tr>
              <td class="font-mono">{d.id}</td>
              <td class="font-mono" style="font-weight: 600;">{d.host}</td>
              <td class="font-mono">{d.port}</td>
              <td><strong>{d.engine}</strong></td>
              <td>{d.version}</td>
              <td><span class="role-pill async">{d.role}</span></td>
              <td style="color: var(--text-muted);">{d.discoveredAt}</td>
              <td>
                <button class="btn btn-primary btn-sm" onclick={() => importDiscoveredWorkload(d.id)}>Import to Nodlance</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</section>
