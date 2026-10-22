<script>
  import {
    activeModal,
    modalContextNode,
    closeModal,
    activeCluster,
    clusters,
    promoteCandidate,
    fenceNode,
    triggerClusterBackup,
    runDiscoveryScan,
    deployNewCluster,
    createLoadBalancer,
    showToast
  } from '../lib/stores.js';

  // --- Wizard State ---
  let wizardStep = $state(1);
  const totalWizardSteps = 5;
  let wizProvider = $state('linux');
  let wizEngine = $state('postgres');
  let wizClusterName = $state('prod-pg16-analytics');
  let wizNodeCount = $state(3);
  let wizPort = $state(5432);
  let isDeploying = $state(false);

  // --- Load Balancer Modal State ---
  let lbName = $state('ProxySQL Production Router');
  let lbType = $state('proxysql');
  let lbClusterId = $state('c1a7001-pg16-prod');
  let lbHost = $state('192.168.10.30');
  let lbListenPort = $state(6033);
  let lbAdminPort = $state(6032);

  function handleLbTypeChange(type) {
    lbType = type;
    if (type === 'proxysql') {
      lbName = 'ProxySQL Core Router';
      lbListenPort = 6033;
      lbAdminPort = 6032;
    } else if (type === 'pgbouncer') {
      lbName = 'PgBouncer Pooler';
      lbListenPort = 6432;
      lbAdminPort = 6433;
    } else if (type === 'keepalived') {
      lbName = 'Keepalived VIP Daemon';
      lbListenPort = 5432;
      lbAdminPort = null;
    } else if (type === 'haproxy') {
      lbName = 'HAProxy TCP Balancer';
      lbListenPort = 5000;
      lbAdminPort = 9000;
    }
  }

  function submitLoadBalancer() {
    const selectedCluster = $clusters.find(c => c.id === lbClusterId);
    createLoadBalancer({
      name: lbName,
      type: lbType,
      clusterId: lbClusterId,
      clusterName: selectedCluster ? selectedCluster.name : 'Target Cluster',
      host: lbHost,
      ip: lbHost,
      listenPort: lbListenPort,
      adminPort: lbAdminPort,
    });
  }

  // --- Failover State ---
  let selectedCandidateId = $state('');
  let promoteToken = $state('tok_prom_' + Math.random().toString(16).slice(2, 10));
  let fenceOldPrimary = $state(true);

  // --- Fencing State ---
  let fenceTargetNodeId = $state('');
  let fenceMechanism = $state('stonith');
  let fenceReason = $state('Unresponsive replication heartbeat > 30s');

  // --- Backup State ---
  let backupClusterId = $state('');
  let backupType = $state('full');
  let backupTarget = $state('s3');

  // --- Scan State ---
  let scanCidr = $state('192.168.10.0/24');

  $effect(() => {
    if ($activeCluster) {
      backupClusterId = $activeCluster.id;
      const firstStandby = $activeCluster.nodes.find(n => n.role !== 'primary');
      if (firstStandby) selectedCandidateId = firstStandby.id;
      if ($modalContextNode) {
        selectedCandidateId = $modalContextNode.id;
        fenceTargetNodeId = $modalContextNode.id;
      } else if ($activeCluster.nodes.length > 0) {
        fenceTargetNodeId = $activeCluster.nodes[0].id;
      }
    }
  });

  function regenToken() {
    promoteToken = 'tok_prom_' + Math.random().toString(16).slice(2, 10) + Math.random().toString(16).slice(2, 6);
    showToast('New cryptographic operator approval token generated.', 'info');
  }

  function handleExecutePromotion() {
    if (!selectedCandidateId) return;
    promoteCandidate(selectedCandidateId);
    closeModal();
  }

  function handleConfirmFence() {
    if (!fenceTargetNodeId) return;
    fenceNode(fenceTargetNodeId, fenceMechanism, fenceReason);
    closeModal();
  }

  function handleTriggerBackup() {
    triggerClusterBackup(backupClusterId, backupType);
    closeModal();
  }

  function handleRunScan() {
    runDiscoveryScan(scanCidr);
    closeModal();
  }

  function copyEnrollCmd() {
    const cmd = 'curl -sSL https://nodlance.internal/install.sh | sudo NODLANCE_TOKEN="nl_agt_enroll_e8d7a1b4" sh';
    navigator.clipboard.writeText(cmd).then(() => {
      showToast('Agent installation command copied to clipboard!', 'success');
    });
  }

  function handleWizardNext() {
    if (wizardStep < totalWizardSteps) {
      wizardStep++;
    } else {
      isDeploying = true;
      setTimeout(() => {
        deployNewCluster({
          clusterName: wizClusterName,
          provider: wizProvider,
          engine: wizEngine,
          nodeCount: wizNodeCount,
          port: wizPort
        });
        isDeploying = false;
        wizardStep = 1;
        closeModal();
      }, 1200);
    }
  }

  function handleWizardBack() {
    if (wizardStep > 1) wizardStep--;
  }
</script>

<!-- 1. Deploy Cluster Wizard Modal -->
{#if $activeModal === 'deployWizard'}
  <div
    class="modal-backdrop open"
    onclick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
    onkeydown={(e) => { if (e.key === 'Escape') closeModal(); }}
    role="dialog"
    tabindex="-1"
  >
    <div class="modal-window modal-lg">
      <div class="modal-header">
        <div class="wizard-header-info">
          <h2 class="modal-title">Deploy Database Cluster</h2>
          <span class="wizard-step-counter">Step {wizardStep} of {totalWizardSteps}</span>
        </div>
        <button class="modal-close-btn" onclick={closeModal}>&times;</button>
      </div>

      <div class="wizard-progress-bar">
        <div class="wizard-progress-fill" style="width: {(wizardStep / totalWizardSteps) * 100}%;"></div>
      </div>

      <div class="modal-body wizard-body">
        {#if wizardStep === 1}
          <div class="wizard-step active">
            <h3 class="step-title">Select Infrastructure Provider</h3>
            <p class="step-desc">Choose where your database cluster instances will be provisioned.</p>
            <div class="provider-cards-grid">
              <label class="provider-option {wizProvider === 'linux' ? 'selected' : ''}">
                <input type="radio" bind:group={wizProvider} value="linux" />
                <div class="provider-card">
                  <svg class="provider-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="14" x="2" y="3" rx="2"/><line x1="8" x2="16" y1="21" y2="21"/><line x1="12" x2="12" y1="17" y2="21"/></svg>
                  <span class="provider-name">Linux Baremetal / VM</span>
                  <span class="provider-info">SSH, systemd unit supervisor, native disk I/O</span>
                </div>
              </label>
              <label class="provider-option {wizProvider === 'docker' ? 'selected' : ''}">
                <input type="radio" bind:group={wizProvider} value="docker" />
                <div class="provider-card">
                  <svg class="provider-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 11h16"/><path d="M4 15h16"/><path d="M4 19h16"/><path d="M7 7h4v4H7z"/></svg>
                  <span class="provider-name">Docker Engine</span>
                  <span class="provider-info">Isolated container runtimes, custom bridge network</span>
                </div>
              </label>
              <label class="provider-option {wizProvider === 'k8s' ? 'selected' : ''}">
                <input type="radio" bind:group={wizProvider} value="k8s" />
                <div class="provider-card">
                  <svg class="provider-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polygon points="12 6 17 15 7 15"/></svg>
                  <span class="provider-name">Kubernetes</span>
                  <span class="provider-info">StatefulSets, PersistentVolumeClaims, Headless service</span>
                </div>
              </label>
            </div>
          </div>
        {:else if wizardStep === 2}
          <div class="wizard-step active">
            <h3 class="step-title">Select Database Engine & Version</h3>
            <p class="step-desc">Choose the relational database engine and target version.</p>
            <div class="engine-cards-grid">
              <label class="engine-option {wizEngine === 'postgres' ? 'selected' : ''}">
                <input type="radio" bind:group={wizEngine} value="postgres" />
                <div class="engine-card">
                  <span class="engine-tag pg">PG</span>
                  <span class="engine-name">PostgreSQL 16 HA</span>
                  <span class="engine-detail">Streaming replication with synchronous commit & slots</span>
                </div>
              </label>
              <label class="engine-option {wizEngine === 'mysql' ? 'selected' : ''}">
                <input type="radio" bind:group={wizEngine} value="mysql" />
                <div class="engine-card">
                  <span class="engine-tag my">MY</span>
                  <span class="engine-name">MySQL 8.4 LTS</span>
                  <span class="engine-detail">Group Replication with GTID auto-positioning</span>
                </div>
              </label>
              <label class="engine-option {wizEngine === 'mariadb' ? 'selected' : ''}">
                <input type="radio" bind:group={wizEngine} value="mariadb" />
                <div class="engine-card">
                  <span class="engine-tag ma">MA</span>
                  <span class="engine-name">MariaDB 11.4 Galera</span>
                  <span class="engine-detail">Multi-master synchronous cluster with wsrep provider</span>
                </div>
              </label>
            </div>
          </div>
        {:else if wizardStep === 3}
          <div class="wizard-step active">
            <h3 class="step-title">Configure Cluster Topology</h3>
            <div class="form-group">
              <label for="wizClusterName" class="form-label">Cluster Identifier</label>
              <input type="text" id="wizClusterName" class="form-input" bind:value={wizClusterName} placeholder="e.g. prod-db-cluster-01" />
            </div>
            <div class="form-row">
              <div class="form-group col-6">
                <label for="wizNodeCount" class="form-label">Node Count (Quorum)</label>
                <select id="wizNodeCount" class="form-input" bind:value={wizNodeCount}>
                  <option value={3}>3 Nodes (1 Primary, 2 Standbys - HA Quorum)</option>
                  <option value={5}>5 Nodes (1 Primary, 4 Standbys - Multi-AZ Quorum)</option>
                  <option value={2}>2 Nodes + 1 Arbiter</option>
                </select>
              </div>
              <div class="form-group col-6">
                <label for="wizPort" class="form-label">Database Port</label>
                <input type="number" id="wizPort" class="form-input" bind:value={wizPort} />
              </div>
            </div>
          </div>
        {:else if wizardStep === 4}
          <div class="wizard-step active">
            <h3 class="step-title">Assign Infrastructure Hosts</h3>
            <p class="step-desc">Designate target hosts from your registered inventory for Primary and Standby roles.</p>
            <div class="assigned-hosts-list">
              <div class="host-assign-row">
                <span class="host-role-badge primary">Primary</span>
                <span class="host-assign-name">srv-node-01.prod (192.168.10.21)</span>
                <span class="host-assign-state text-emerald">Verified</span>
              </div>
              <div class="host-assign-row">
                <span class="host-role-badge sync">Standby 1</span>
                <span class="host-assign-name">srv-node-02.prod (192.168.10.22)</span>
                <span class="host-assign-state text-emerald">Verified</span>
              </div>
              <div class="host-assign-row">
                <span class="host-role-badge async">Standby 2</span>
                <span class="host-assign-name">srv-node-03.prod (192.168.10.23)</span>
                <span class="host-assign-state text-emerald">Verified</span>
              </div>
            </div>
          </div>
        {:else if wizardStep === 5}
          <div class="wizard-step active">
            <h3 class="step-title">Guarded Execution Plan (with LIFO Rollback)</h3>
            <p class="step-desc">Review the ordered provisioning plan steps. If any step fails, Nodlance executes strict reverse LIFO rollback to leave hosts untouched.</p>
            <div class="plan-steps-preview">
              <div class="plan-step-item">
                <span class="step-seq">1</span>
                <span class="step-title-text">Pre-flight Environment Validation & Port Verification</span>
              </div>
              <div class="plan-step-item">
                <span class="step-seq">2</span>
                <span class="step-title-text">Initialize Primary Node Data Directory & SSL Certificates</span>
              </div>
              <div class="plan-step-item">
                <span class="step-seq">3</span>
                <span class="step-title-text">Configure Replication Slots & pg_hba.conf Authentication</span>
              </div>
              <div class="plan-step-item">
                <span class="step-seq">4</span>
                <span class="step-title-text">Execute Standby pg_basebackup Clone & Start Services</span>
              </div>
              <div class="plan-step-item">
                <span class="step-seq">5</span>
                <span class="step-title-text">Establish Epoch #1 & Verify Quorum Consensus</span>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        {#if wizardStep > 1}
          <button class="btn btn-secondary" onclick={handleWizardBack}>Back</button>
        {/if}
        <button class="btn btn-primary" onclick={handleWizardNext} disabled={isDeploying}>
          {isDeploying ? 'Deploying...' : wizardStep === totalWizardSteps ? 'Execute Guarded Plan' : `Next: Step ${wizardStep + 1}`}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- 2. Failover Modal -->
{#if $activeModal === 'failover'}
  <div
    class="modal-backdrop open"
    onclick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
    onkeydown={(e) => { if (e.key === 'Escape') closeModal(); }}
    role="dialog"
    tabindex="-1"
  >
    <div class="modal-window">
      <div class="modal-header">
        <h2 class="modal-title">Promote Standby Node (Guarded Failover)</h2>
        <button class="modal-close-btn" onclick={closeModal}>&times;</button>
      </div>
      <div class="modal-body">
        <div class="alert-box warning">
          <svg class="alert-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><line x1="12" x2="12" y1="9" y2="13"/></svg>
          <div class="alert-content">
            <div class="alert-title">Operator Approval Required (Two-Man Rule)</div>
            <div class="alert-text">Promoting a replica triggers STONITH fencing on the existing primary to prevent split-brain writes and increments the cluster epoch.</div>
          </div>
        </div>
        <div class="form-group">
          <label for="promoteCandidateSelect" class="form-label">Select Standby Node to Promote</label>
          <select id="promoteCandidateSelect" class="form-input" bind:value={selectedCandidateId}>
            {#if $activeCluster}
              {#each $activeCluster.nodes.filter(n => n.role !== 'primary') as n}
                <option value={n.id}>{n.id} ({n.ip} - Lag: {n.lagMs}ms)</option>
              {/each}
            {/if}
          </select>
        </div>
        <div class="form-group">
          <label for="promoteTokenInput" class="form-label">Operator Approval Token</label>
          <div class="input-with-action">
            <input type="text" id="promoteTokenInput" class="form-input font-mono" readonly value={promoteToken} />
            <button class="btn btn-secondary btn-sm" onclick={regenToken}>Generate Token</button>
          </div>
        </div>
        <div class="form-group">
          <label class="form-checkbox-label">
            <input type="checkbox" bind:checked={fenceOldPrimary} />
            <span>Automatically trigger STONITH fencing against old primary node</span>
          </label>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={closeModal}>Cancel</button>
        <button class="btn btn-danger" onclick={handleExecutePromotion}>Confirm & Promote Candidate</button>
      </div>
    </div>
  </div>
{/if}

<!-- 3. Fencing Modal -->
{#if $activeModal === 'fence'}
  <div
    class="modal-backdrop open"
    onclick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
    onkeydown={(e) => { if (e.key === 'Escape') closeModal(); }}
    role="dialog"
    tabindex="-1"
  >
    <div class="modal-window">
      <div class="modal-header">
        <h2 class="modal-title">Execute Verifiable Node Fencing (STONITH)</h2>
        <button class="modal-close-btn" onclick={closeModal}>&times;</button>
      </div>
      <div class="modal-body">
        <div class="alert-box danger">
          <svg class="alert-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10"/></svg>
          <div class="alert-content">
            <div class="alert-title">Irrevocable Node Isolation</div>
            <div class="alert-text">Target node will be instantly cut from client traffic and storage to protect data integrity.</div>
          </div>
        </div>
        <div class="form-group">
          <label for="fenceNodeSelect" class="form-label">Target Node</label>
          <select id="fenceNodeSelect" class="form-input" bind:value={fenceTargetNodeId}>
            {#if $activeCluster}
              {#each $activeCluster.nodes as n}
                <option value={n.id}>{n.id} ({n.ip}:{n.port}) - {n.role.toUpperCase()}</option>
              {/each}
            {/if}
          </select>
        </div>
        <div class="form-group">
          <label for="fenceMechanismSelect" class="form-label">Fencing Mechanism</label>
          <select id="fenceMechanismSelect" class="form-input" bind:value={fenceMechanism}>
            <option value="stonith">STONITH (Shoot The Other Node In The Head - Hard Power Off)</option>
            <option value="network">Network Isolation (iptables / Security Group Drop)</option>
            <option value="storage">Storage Revocation (SCSI Reservation Revoke)</option>
          </select>
        </div>
        <div class="form-group">
          <label for="fenceReasonInput" class="form-label">Reason for Fencing</label>
          <input type="text" id="fenceReasonInput" class="form-input" bind:value={fenceReason} />
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={closeModal}>Cancel</button>
        <button class="btn btn-danger" onclick={handleConfirmFence}>Isolate & Fence Node</button>
      </div>
    </div>
  </div>
{/if}

<!-- 4. Backup Modal -->
{#if $activeModal === 'backup'}
  <div
    class="modal-backdrop open"
    onclick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
    onkeydown={(e) => { if (e.key === 'Escape') closeModal(); }}
    role="dialog"
    tabindex="-1"
  >
    <div class="modal-window">
      <div class="modal-header">
        <h2 class="modal-title">Trigger On-Demand Cluster Backup</h2>
        <button class="modal-close-btn" onclick={closeModal}>&times;</button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label for="backupClusterSelect" class="form-label">Target Cluster</label>
          <select id="backupClusterSelect" class="form-input" bind:value={backupClusterId}>
            {#each $clusters as c}
              <option value={c.id}>{c.name}</option>
            {/each}
          </select>
        </div>
        <div class="form-group">
          <label for="backupTypeSelect" class="form-label">Backup Type</label>
          <select id="backupTypeSelect" class="form-input" bind:value={backupType}>
            <option value="full">Full Physical Snapshot (pg_basebackup / xtrabackup)</option>
            <option value="incremental">Incremental WAL / Binlog Stream</option>
            <option value="logical">Logical Schema & Data Dump</option>
          </select>
        </div>
        <div class="form-group">
          <label for="backupTargetSelect" class="form-label">Storage Target</label>
          <select id="backupTargetSelect" class="form-input" bind:value={backupTarget}>
            <option value="s3">Primary Object Store (MinIO / AWS S3 - eu-central)</option>
            <option value="zfs">Local ZFS Encrypted Storage Pool (/mnt/backups)</option>
          </select>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={closeModal}>Cancel</button>
        <button class="btn btn-primary" onclick={handleTriggerBackup}>Start Backup Job</button>
      </div>
    </div>
  </div>
{/if}

<!-- 5. Scan Modal -->
{#if $activeModal === 'scan'}
  <div
    class="modal-backdrop open"
    onclick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
    onkeydown={(e) => { if (e.key === 'Escape') closeModal(); }}
    role="dialog"
    tabindex="-1"
  >
    <div class="modal-window">
      <div class="modal-header">
        <h2 class="modal-title">Run Infrastructure Workload Discovery</h2>
        <button class="modal-close-btn" onclick={closeModal}>&times;</button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label for="scanCidrInput" class="form-label">Target Subnet / Namespace</label>
          <input type="text" id="scanCidrInput" class="form-input" bind:value={scanCidr} />
        </div>
        <div class="form-group">
          <span class="form-label">Probe Ports</span>
          <div class="checkbox-group">
            <label class="form-checkbox-label"><input type="checkbox" checked /> 5432 (PostgreSQL)</label>
            <label class="form-checkbox-label"><input type="checkbox" checked /> 3306 (MySQL / MariaDB)</label>
            <label class="form-checkbox-label"><input type="checkbox" checked /> 4567 (Galera wsrep)</label>
          </div>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={closeModal}>Cancel</button>
        <button class="btn btn-primary" onclick={handleRunScan}>Start Discovery Scan</button>
      </div>
    </div>
  </div>
{/if}

<!-- 6. Enroll Agent Modal -->
{#if $activeModal === 'enroll'}
  <div
    class="modal-backdrop open"
    onclick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
    onkeydown={(e) => { if (e.key === 'Escape') closeModal(); }}
    role="dialog"
    tabindex="-1"
  >
    <div class="modal-window">
      <div class="modal-header">
        <h2 class="modal-title">Enroll New Host Agent</h2>
        <button class="modal-close-btn" onclick={closeModal}>&times;</button>
      </div>
      <div class="modal-body">
        <p class="step-desc">Run this single command as root on your target Linux host to install the lightweight Nodlance telemetry daemon and securely connect to the control plane.</p>
        <div class="form-group">
          <span class="form-label">One-Line Agent Install Command</span>
          <div class="code-box">
            <code>curl -sSL https://nodlance.internal/install.sh | sudo NODLANCE_TOKEN="nl_agt_enroll_e8d7a1b4" sh</code>
            <button class="btn btn-secondary btn-sm" onclick={copyEnrollCmd}>Copy</button>
          </div>
        </div>
        <div class="enroll-status-box">
          <span class="pulse-indicator"></span>
          <span>Waiting for inbound host heartbeat from target node...</span>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={closeModal}>Close</button>
      </div>
    </div>
  </div>
{/if}

<!-- 7. Deploy Load Balancer Modal -->
{#if $activeModal === 'deployLoadBalancer'}
  <div
    class="modal-backdrop open"
    onclick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
    onkeydown={(e) => { if (e.key === 'Escape') closeModal(); }}
    role="dialog"
    tabindex="-1"
  >
    <div class="modal-window">
      <div class="modal-header">
        <h2 class="modal-title">Deploy Database Load Balancer / VIP</h2>
        <button class="modal-close-btn" onclick={closeModal}>&times;</button>
      </div>
      <div class="modal-body">
        <p class="step-desc">Deploy and configure high-performance traffic routing, connection pooling, or Virtual IP failover for your database fleet.</p>

        <div class="form-group">
          <label class="form-label" for="lb-type-select">Router Technology</label>
          <div class="engine-selector">
            <button
              class="engine-card {lbType === 'proxysql' ? 'selected' : ''}"
              onclick={() => handleLbTypeChange('proxysql')}
              type="button"
            >
              <span class="engine-title">ProxySQL</span>
              <span class="engine-sub">MySQL R/W Split & Hostgroups</span>
            </button>
            <button
              class="engine-card {lbType === 'pgbouncer' ? 'selected' : ''}"
              onclick={() => handleLbTypeChange('pgbouncer')}
              type="button"
            >
              <span class="engine-title">PgBouncer</span>
              <span class="engine-sub">PostgreSQL Connection Pooling</span>
            </button>
            <button
              class="engine-card {lbType === 'keepalived' ? 'selected' : ''}"
              onclick={() => handleLbTypeChange('keepalived')}
              type="button"
            >
              <span class="engine-title">Keepalived</span>
              <span class="engine-sub">Virtual IP (VRRP) Failover</span>
            </button>
            <button
              class="engine-card {lbType === 'haproxy' ? 'selected' : ''}"
              onclick={() => handleLbTypeChange('haproxy')}
              type="button"
            >
              <span class="engine-title">HAProxy</span>
              <span class="engine-sub">Layer-4 TCP Load Balancer</span>
            </button>
          </div>
        </div>

        <div class="form-group">
          <label class="form-label" for="lb-name-input">Instance Name</label>
          <input id="lb-name-input" class="form-input" type="text" bind:value={lbName} />
        </div>

        <div class="form-group">
          <label class="form-label" for="lb-cluster-select">Target Database Cluster</label>
          <select id="lb-cluster-select" class="form-input" bind:value={lbClusterId}>
            {#each $clusters as c}
              <option value={c.id}>{c.name} ({c.engineName})</option>
            {/each}
          </select>
        </div>

        <div class="form-row">
          <div class="form-group flex-1">
            <label class="form-label" for="lb-host-input">Host Node / IP</label>
            <input id="lb-host-input" class="form-input" type="text" bind:value={lbHost} />
          </div>
          <div class="form-group" style="width: 140px;">
            <label class="form-label" for="lb-listen-port">Listen Port</label>
            <input id="lb-listen-port" class="form-input" type="number" bind:value={lbListenPort} />
          </div>
          {#if lbType === 'proxysql' || lbType === 'pgbouncer'}
            <div class="form-group" style="width: 140px;">
              <label class="form-label" for="lb-admin-port">Admin Port</label>
              <input id="lb-admin-port" class="form-input" type="number" bind:value={lbAdminPort} />
            </div>
          {/if}
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={closeModal}>Cancel</button>
        <button class="btn btn-primary" onclick={submitLoadBalancer}>
          Deploy Load Balancer
        </button>
      </div>
    </div>
  </div>
{/if}
