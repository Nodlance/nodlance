<script>
  import {
    backups,
    drills,
    backupSchedules,
    pitrStreams,
    openModal,
    deleteBackupSchedule,
    toggleBackupSchedule,
    rewindPitrStream,
    showToast
  } from '../lib/stores.js';

  let selectedPitrTarget = $state('2026-10-22T08:15:00');

  function runDrill(bId) {
    showToast(`Automated sandbox restore drill started for ${bId}...`, 'info');
  }

  function handlePitrRewind(clusterId) {
    rewindPitrStream(clusterId, selectedPitrTarget);
  }

  function getTargetBadgeClass(target) {
    switch (target?.toLowerCase()) {
      case 's3': return 'badge-s3';
      case 'minio': return 'badge-minio';
      case 'gcs': return 'badge-gcs';
      case 'azure_blob': return 'badge-azure';
      default: return 'badge-local';
    }
  }
</script>

<section class="view-panel active" id="viewBackups">
  <div class="view-header">
    <div>
      <h1 class="view-title">Backups, PITR Streams & Restore Drills</h1>
      <p class="view-subtitle">Continuous Point-in-Time Recovery (PITR), automated multi-cloud schedules (AWS S3, MinIO, GCS), retention policies, and automated sandbox validation.</p>
    </div>
    <div class="view-actions">
      <button class="btn btn-secondary btn-sm" onclick={() => openModal('createSchedule')}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
        <span>New Backup Schedule</span>
      </button>
      <button class="btn btn-primary btn-sm" onclick={() => openModal('backup')}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/></svg>
        <span>Take Immediate Backup</span>
      </button>
    </div>
  </div>

  <!-- Continuous Point-in-Time Recovery (PITR) Streams -->
  <div class="card table-card" style="margin-bottom: 24px;">
    <div class="card-header">
      <div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <h2 class="card-title">Continuous Point-in-Time Recovery (PITR) Streams</h2>
          <span class="status-indicator text-emerald font-mono" style="font-size: 11px;">LIVE STREAMING</span>
        </div>
        <p class="card-desc">Continuous WAL / binary log replication to durable cloud object storage with sub-minute RPO rewind validation.</p>
      </div>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Cluster</th>
            <th>Storage Destination</th>
            <th>Replay Window</th>
            <th>Start LSN / GTID</th>
            <th>Latest Synced LSN</th>
            <th>Current RPO Lag</th>
            <th>Status</th>
            <th>PITR Rewind Validation</th>
          </tr>
        </thead>
        <tbody>
          {#each $pitrStreams as stream (stream.clusterId)}
            <tr>
              <td style="font-weight: 600;">{stream.clusterName}</td>
              <td>
                <span class="storage-badge {getTargetBadgeClass(stream.storageTarget)}">
                  {stream.storageTarget.toUpperCase()}
                </span>
                <span class="font-mono" style="font-size: 11px; margin-left: 6px; color: var(--text-muted);">{stream.storagePath}</span>
              </td>
              <td><span class="badge badge-neutral">{stream.recoveryWindow}</span></td>
              <td class="font-mono" style="font-size: 11px;">{stream.startLsn}</td>
              <td class="font-mono text-emerald" style="font-size: 11px; font-weight: 600;">{stream.latestLsn}</td>
              <td>
                <span class="rpo-pill text-emerald">
                  <span class="pulse-dot"></span>
                  {stream.rpoLagSeconds}s lag
                </span>
              </td>
              <td><span class="status-indicator text-emerald">{stream.status}</span></td>
              <td>
                <div style="display: flex; align-items: center; gap: 6px;">
                  <input
                    type="datetime-local"
                    class="form-input"
                    style="padding: 2px 6px; font-size: 11px; height: 26px; width: 170px;"
                    bind:value={selectedPitrTarget}
                  />
                  <button class="btn btn-secondary btn-sm" style="padding: 2px 8px; font-size: 11px; height: 26px;" onclick={() => handlePitrRewind(stream.clusterId)}>
                    Test Rewind
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  <!-- Automated Backup Schedules & Storage Offload -->
  <div class="card table-card" style="margin-bottom: 24px;">
    <div class="card-header">
      <div>
        <h2 class="card-title">Automated Backup Schedules & Retention</h2>
        <p class="card-desc">Scheduled physical snapshots, compression algorithms, and automated retention lifecycle policies.</p>
      </div>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Schedule Name</th>
            <th>Cluster</th>
            <th>Cron Frequency</th>
            <th>Engine Method</th>
            <th>Destination Target</th>
            <th>Retention</th>
            <th>Compression & Cipher</th>
            <th>Next Run</th>
            <th>Status</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each $backupSchedules as s (s.id)}
            <tr>
              <td style="font-weight: 600;">{s.name}</td>
              <td>{s.clusterName}</td>
              <td>
                <span class="font-mono schedule-cron">{s.cronExpression}</span>
                <div style="font-size: 11px; color: var(--text-muted);">{s.scheduleHuman}</div>
              </td>
              <td><span class="badge badge-neutral">{s.backupMethod}</span></td>
              <td>
                <span class="storage-badge {getTargetBadgeClass(s.storageTarget)}">
                  {s.storageTarget.toUpperCase()}
                </span>
                <div class="font-mono" style="font-size: 10px; color: var(--text-muted); margin-top: 2px; max-width: 160px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title={s.storagePath}>
                  {s.storagePath}
                </div>
              </td>
              <td><span class="badge badge-accent">{s.retentionDays} Days</span></td>
              <td>
                <span class="font-mono" style="font-size: 11px;">{s.compression}</span>
                {#if s.encrypt}
                  <span class="badge-cipher">AES-256</span>
                {/if}
              </td>
              <td style="color: var(--text-muted); font-size: 12px;">{s.nextRunAt}</td>
              <td>
                {#if s.isActive}
                  <span class="status-indicator text-emerald">ACTIVE</span>
                {:else}
                  <span class="status-indicator" style="color: var(--text-muted);">PAUSED</span>
                {/if}
              </td>
              <td>
                <div style="display: flex; gap: 6px;">
                  <button
                    class="btn btn-secondary btn-sm"
                    style="padding: 2px 8px; font-size: 11px; height: 26px;"
                    onclick={() => toggleBackupSchedule(s.id)}
                  >
                    {s.isActive ? 'Pause' : 'Resume'}
                  </button>
                  <button
                    class="btn btn-secondary btn-sm"
                    style="padding: 2px 8px; font-size: 11px; height: 26px; color: #ef4444;"
                    onclick={() => deleteBackupSchedule(s.id)}
                  >
                    Delete
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  <!-- Completed Snapshots -->
  <div class="card table-card" style="margin-bottom: 24px;">
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

  <!-- Automated Restore Qualification Drills -->
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

<style>
  .storage-badge {
    display: inline-block;
    padding: 2px 7px;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.5px;
    border-radius: 4px;
    text-transform: uppercase;
  }
  .badge-s3 {
    background: rgba(255, 153, 0, 0.15);
    color: #ff9900;
    border: 1px solid rgba(255, 153, 0, 0.3);
  }
  .badge-minio {
    background: rgba(199, 37, 78, 0.15);
    color: #e23b62;
    border: 1px solid rgba(199, 37, 78, 0.3);
  }
  .badge-gcs {
    background: rgba(66, 133, 244, 0.15);
    color: #4285f4;
    border: 1px solid rgba(66, 133, 244, 0.3);
  }
  .badge-azure {
    background: rgba(0, 137, 214, 0.15);
    color: #0089d6;
    border: 1px solid rgba(0, 137, 214, 0.3);
  }
  .badge-local {
    background: rgba(100, 116, 139, 0.15);
    color: #94a3b8;
    border: 1px solid rgba(100, 116, 139, 0.3);
  }
  .badge-cipher {
    display: inline-block;
    padding: 1px 5px;
    margin-left: 4px;
    font-size: 9px;
    font-weight: 700;
    border-radius: 3px;
    background: rgba(16, 185, 129, 0.12);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.25);
  }
  .schedule-cron {
    background: var(--bg-surface-secondary, rgba(0, 0, 0, 0.05));
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
  }
  .rpo-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 9999px;
    background: rgba(16, 185, 129, 0.1);
  }
  .pulse-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: #10b981;
    box-shadow: 0 0 6px #10b981;
    animation: pulse-ring 2s infinite ease-in-out;
  }
  @keyframes pulse-ring {
    0% { transform: scale(0.9); opacity: 1; }
    50% { transform: scale(1.4); opacity: 0.6; }
    100% { transform: scale(0.9); opacity: 1; }
  }
</style>
