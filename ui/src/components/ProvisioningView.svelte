<script>
  import { plans, openModal, showToast } from '../lib/stores.js';

  function viewPlanDetails(id) {
    showToast(`Viewing 5-step execution & LIFO rollback plan for ${id}`, 'info');
  }
</script>

<section class="view-panel active" id="viewProvisioning">
  <div class="view-header">
    <div>
      <h1 class="view-title">Guarded Cluster Provisioning</h1>
      <p class="view-subtitle">Multi-step cluster deployment with LIFO reverse rollback safety guarantees across Baremetal, Docker, and Kubernetes.</p>
    </div>
    <div class="view-actions">
      <button class="btn btn-primary btn-sm" onclick={() => openModal('deployWizard')}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
        <span>New Cluster Wizard</span>
      </button>
    </div>
  </div>

  <div class="card table-card">
    <div class="card-header">
      <div>
        <h2 class="card-title">Execution Plans & Rollback Logs</h2>
        <p class="card-desc">Stateful provisioning plans, step tracking, and verified rollbacks.</p>
      </div>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Plan ID</th>
            <th>Cluster Target</th>
            <th>Provider</th>
            <th>Engine</th>
            <th>Total Steps</th>
            <th>Status</th>
            <th>Created At</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each $plans as p (p.id)}
            <tr>
              <td class="font-mono">{p.id}</td>
              <td style="font-weight: 600;">{p.cluster}</td>
              <td>{p.provider}</td>
              <td>{p.engine}</td>
              <td class="font-mono">{p.steps}</td>
              <td><span class="status-indicator text-emerald">{p.status}</span></td>
              <td style="color: var(--text-muted);">{p.created}</td>
              <td>
                <button class="btn btn-secondary btn-sm" onclick={() => viewPlanDetails(p.id)}>View Steps</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</section>
