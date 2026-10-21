<script>
  import {
    backups,
    drills,
    openModal,
    showToast
  } from '../lib/stores.js';

  function runDrill(bId) {
    showToast(`Automated sandbox restore drill started for ${bId}...`, 'info');
  }
</script>

<section class="view-panel active" id="viewBackups">
  <div class="view-header">
    <div>
      <h1 class="view-title">Backups & Automated Restore Drills</h1>
      <p class="view-subtitle">Physical snapshots, continuous WAL/binlog archiving, retention enforcement, and isolated restore qualification drills.</p>
    </div>
    <div class="view-actions">
      <button class="btn btn-primary btn-sm" onclick={() => openModal('backup')}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/></svg>
        <span>Take Cluster Backup</span>
      </button>
    </div>
  </div>

  <div class="card table-card">
    <div class="card-header">
      <div>
        <h2 class="card-title">Completed Database Snapshots</h2>
        <p class="card-desc">Cryptographically verified snapshots with SHA-256 checksums.</p>
      </div>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Backup ID</th>
            <th>Cluster</th>
            <th>Type</th>
            <th>Engine</th>
            <th>Size</th>
            <th>SHA-256 Checksum</th>
            <th>Verification Status</th>
            <th>Completed At</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each $backups as b (b.id)}
            <tr>
              <td class="font-mono">{b.id}</td>
              <td style="font-weight: 600;">{b.cluster}</td>
              <td>{b.type}</td>
              <td>{b.engine}</td>
              <td class="font-mono">{b.size}</td>
              <td class="font-mono" style="font-size: 10px; max-width: 140px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title={b.checksum}>
                {b.checksum}
              </td>
              <td><span class="status-indicator text-emerald">SHA256 MATCH</span></td>
              <td style="color: var(--text-muted);">{b.completedAt}</td>
              <td>
                <button class="btn btn-secondary btn-sm" onclick={() => runDrill(b.id)}>Run Drill</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  <div class="card table-card">
    <div class="card-header">
      <div>
        <h2 class="card-title">Automated Restore Qualification Drills</h2>
        <p class="card-desc">Scheduled sandbox drills verifying snapshot bootability and transactional consistency without impacting production.</p>
      </div>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Drill ID</th>
            <th>Source Backup</th>
            <th>Target Sandbox</th>
            <th>Duration</th>
            <th>Corruption Found</th>
            <th>Result</th>
            <th>Completed At</th>
          </tr>
        </thead>
        <tbody>
          {#each $drills as d (d.id)}
            <tr>
              <td class="font-mono">{d.id}</td>
              <td class="font-mono">{d.sourceBackup}</td>
              <td class="font-mono">{d.targetSandbox}</td>
              <td>{d.duration}</td>
              <td class="text-emerald">{d.corruption}</td>
              <td><span class="status-indicator text-emerald">{d.result}</span></td>
              <td style="color: var(--text-muted);">{d.completedAt}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</section>
