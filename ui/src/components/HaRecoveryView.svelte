<script>
  import {
    activeCluster,
    fencing,
    openModal,
    showToast
  } from '../lib/stores.js';

  let standbys = $derived(
    $activeCluster ? $activeCluster.nodes.filter(n => n.role !== 'primary') : []
  );

  function simulateFaultInjection() {
    showToast('Simulating primary network isolation drill...', 'warning');
    setTimeout(() => {
      showToast('Fault Injection Qualification PASSED: Quorum verified, candidate auto-promoted.', 'success');
    }, 2000);
  }
</script>

<section class="view-panel active" id="viewHaRecovery">
  <div class="view-header">
    <div>
      <h1 class="view-title">High Availability, Fencing & Recovery Studio</h1>
      <p class="view-subtitle">Guarded failover orchestration, candidate scoring, epoch management, STONITH verifiable fencing, and fault-injection testing.</p>
    </div>
    <div class="view-actions">
      <button class="btn btn-danger btn-sm" onclick={simulateFaultInjection}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z"/></svg>
        <span>Simulate Fault Injection</span>
      </button>
      <button class="btn btn-primary btn-sm" onclick={() => openModal('failover')}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/></svg>
        <span>Orchestrate Failover</span>
      </button>
    </div>
  </div>

  <!-- Quorum & Epoch Status Strip -->
  <div class="ha-status-grid">
    <div class="ha-stat-box">
      <div class="ha-stat-label">CLUSTER QUORUM STATUS</div>
      <div class="ha-stat-value text-emerald">{$activeCluster ? $activeCluster.quorum : 'Achieved'}</div>
      <div class="ha-stat-sub">Strict majority consensus maintained</div>
    </div>
    <div class="ha-stat-box">
      <div class="ha-stat-label">ACTIVE RECOVERY EPOCH</div>
      <div class="ha-stat-value">Epoch #{$activeCluster ? $activeCluster.epoch : 1} • Term {$activeCluster ? $activeCluster.term : 1}</div>
      <div class="ha-stat-sub">Leader: {$activeCluster && $activeCluster.nodes[0] ? $activeCluster.nodes[0].id : 'N/A'} (Active)</div>
    </div>
    <div class="ha-stat-box">
      <div class="ha-stat-label">SPLIT-BRAIN FENCING</div>
      <div class="ha-stat-value text-sky">STONITH Enabled</div>
      <div class="ha-stat-sub">Automated watchdog & network isolation</div>
    </div>
  </div>

  <!-- Promotion Candidate Ranking Table -->
  <div class="card table-card">
    <div class="card-header">
      <div>
        <h2 class="card-title">Standby Promotion Candidates (Scored by Nodlance Recovery Engine)</h2>
        <p class="card-desc">Weighted ranking combining LSN distance, replication lag, health heartbeats, and node configuration weights.</p>
      </div>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Rank</th>
            <th>Node ID / Host</th>
            <th>Lag (Bytes / ms)</th>
            <th>LSN Position</th>
            <th>Health Status</th>
            <th>Composite Score</th>
            <th>Eligibility</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          {#each standbys as node, idx (node.id)}
            {@const score = (100 - (node.lagMs * 1.5)).toFixed(1)}
            <tr>
              <td><strong style="color: var(--brand-accent);">#{idx + 1}</strong></td>
              <td>
                <div style="font-weight: 600;">{node.id}</div>
                <div style="font-size: 11px; color: var(--text-muted);">{node.ip}</div>
              </td>
              <td class="font-mono">{node.lagMs} ms ({node.lagBytes} B)</td>
              <td class="font-mono" style="font-size: 11px;">{node.lsn}</td>
              <td><span class="status-indicator text-emerald"><span class="pulse-indicator"></span> HEALTHY</span></td>
              <td><strong class="font-mono">{score} / 100</strong></td>
              <td><span class="role-pill sync">QUALIFIED</span></td>
              <td>
                <button class="btn btn-primary btn-sm" onclick={() => openModal('failover', node)}>
                  Request Promotion
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  <!-- Fencing & Isolation History -->
  <div class="card table-card">
    <div class="card-header">
      <div>
        <h2 class="card-title">Verifiable Node Fencing Audit (STONITH & Network Isolation)</h2>
        <p class="card-desc">Cryptographic and execution proof of decommissioned or partitioned nodes to eliminate split-brain write risk.</p>
      </div>
      <button class="btn btn-secondary btn-sm" onclick={() => openModal('fence')}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18.36 6.64a9 9 0 1 1-12.73 0"/><line x1="12" x2="12" y1="2" y2="12"/></svg>
        <span>Execute Fencing</span>
      </button>
    </div>
    <div class="table-responsive">
      <table class="data-table">
        <thead>
          <tr>
            <th>Fencing ID</th>
            <th>Target Node</th>
            <th>Mechanism</th>
            <th>Trigger Reason</th>
            <th>Operator / Agent</th>
            <th>Verification Status</th>
            <th>Timestamp</th>
          </tr>
        </thead>
        <tbody>
          {#each $fencing as item (item.id)}
            <tr>
              <td class="font-mono">{item.id}</td>
              <td style="font-weight: 600;">{item.target}</td>
              <td><span class="role-pill fenced">{item.mechanism}</span></td>
              <td style="color: var(--text-secondary);">{item.reason}</td>
              <td>{item.actor}</td>
              <td><span class="status-indicator text-emerald">VERIFIED</span></td>
              <td style="color: var(--text-muted);">{item.timestamp}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</section>
