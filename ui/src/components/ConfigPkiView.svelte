<script>
  import {
    configTemplates,
    pkiCertificates,
    clusters,
    showToast,
    revokeCertificate,
    issueCertificate,
    updateConfigTemplate
  } from '../lib/stores.js';

  let activeTab = $state('config'); // 'config' | 'pki'
  let selectedTemplateId = $state('tmpl_pg16_prod');
  let isEditingTemplate = $state(false);
  let editableContent = $state('');

  // PKI Modal State
  let showIssueModal = $state(false);
  let newCertCn = $state('');
  let newCertClusterId = $state('c1a7001-pg16-prod');
  let newCertType = $state('server');
  let newCertValidity = $state(365);

  // Active Template
  let currentTemplate = $derived(
    $configTemplates.find(t => t.id === selectedTemplateId) || $configTemplates[0]
  );

  // Diff sample simulation based on active template
  let sampleDiffs = $derived.by(() => {
    if (!currentTemplate) return [];
    if (currentTemplate.engine === 'postgres') {
      return [
        {
          param: 'shared_buffers',
          current: '8GB',
          target: '16GB',
          restartRequired: true,
          unit: 'RAM buffer',
          rationale: 'Increased host memory detected (64GB host allocated).'
        },
        {
          param: 'work_mem',
          current: '32MB',
          target: '64MB',
          restartRequired: false,
          unit: 'Session RAM',
          rationale: 'Optimized for complex analytical joins.'
        },
        {
          param: 'max_connections',
          current: '200',
          target: '300',
          restartRequired: true,
          unit: 'Slots',
          rationale: 'Connection ceiling scaled for application pooler.'
        },
        {
          param: 'effective_io_concurrency',
          current: '100',
          target: '200',
          restartRequired: false,
          unit: 'IOPS',
          rationale: 'NVMe SSD storage profile aligned.'
        },
        {
          param: 'ssl',
          current: 'off',
          target: 'on',
          restartRequired: false,
          unit: 'mTLS',
          rationale: 'Enforcing end-to-end client/server encryption.'
        }
      ];
    } else if (currentTemplate.engine === 'mysql') {
      return [
        {
          param: 'innodb_buffer_pool_size',
          current: '16GB',
          target: '24GB',
          restartRequired: true,
          unit: 'InnoDB RAM',
          rationale: 'Expand hot data cache to 75% available RAM.'
        },
        {
          param: 'max_connections',
          current: '300',
          target: '500',
          restartRequired: false,
          unit: 'Slots',
          rationale: 'Dynamically scaled via SET GLOBAL max_connections.'
        },
        {
          param: 'require_secure_transport',
          current: 'OFF',
          target: 'ON',
          restartRequired: false,
          unit: 'Security',
          rationale: 'Block unencrypted plaintext MySQL connections.'
        }
      ];
    } else {
      return [
        {
          param: 'wsrep_sst_method',
          current: 'rsync',
          target: 'mariabackup',
          restartRequired: true,
          unit: 'Galera SST',
          rationale: 'Zero-blocking non-blocking State Snapshot Transfer.'
        },
        {
          param: 'wsrep_provider_options',
          current: 'socket.ssl=off',
          target: 'socket.ssl=on',
          restartRequired: true,
          unit: 'Replication TLS',
          rationale: 'Secure replication traffic between multi-master nodes.'
        }
      ];
    }
  });

  function startEditTemplate() {
    editableContent = currentTemplate.content;
    isEditingTemplate = true;
  }

  function saveTemplate() {
    updateConfigTemplate(currentTemplate.id, editableContent);
    isEditingTemplate = false;
  }

  function handleRollout() {
    const restartCount = sampleDiffs.filter(d => d.restartRequired).length;
    if (restartCount > 0) {
      showToast(
        `Initiating rolling reload & safe-restart across nodes (${restartCount} parameters require rolling daemon restart with leader-handoff).`,
        'warning'
      );
    } else {
      showToast('Initiating zero-downtime dynamic reload (pg_reload_conf / SET GLOBAL)...', 'success');
    }
  }

  function handleIssueCert(e) {
    e.preventDefault();
    if (!newCertCn.trim()) {
      showToast('Common Name (CN) is required.', 'error');
      return;
    }
    const targetCluster = $clusters.find(c => c.id === newCertClusterId);
    issueCertificate({
      clusterId: newCertClusterId,
      clusterName: targetCluster ? targetCluster.name : 'Cluster',
      commonName: newCertCn.trim(),
      certType: newCertType,
      validityDays: parseInt(newCertValidity, 10)
    });
    showIssueModal = false;
    newCertCn = '';
  }

  function handleRevoke(cert) {
    if (confirm(`Revoke certificate for "${cert.commonName}"? This cannot be undone.`)) {
      revokeCertificate(cert.id);
    }
  }

  function downloadCertPem(cert) {
    const pem = `-----BEGIN CERTIFICATE-----
MIICljCCAX4CCQDU3g... [Nodlance Managed PKI]
Subject: CN=${cert.commonName}
SHA-256 Fingerprint: ${cert.fingerprintSha256}
-----END CERTIFICATE-----`;
    const blob = new Blob([pem], { type: 'application/x-pem-file' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${cert.commonName}.crt`;
    a.click();
    URL.revokeObjectURL(url);
    showToast(`Downloaded PEM certificate for ${cert.commonName}`, 'info');
  }
</script>

<div class="config-pki-container">
  <!-- Header Bar -->
  <header class="view-header">
    <div class="header-content">
      <div class="title-with-pill">
        <h1>Configuration Manager & Built-in mTLS PKI CA</h1>
        <span class="engine-badge mTLS">X.509 PKI</span>
        <span class="engine-badge diff">Rolling Diff</span>
      </div>
      <p class="subtitle">
        Declarative database parameter management, dynamic reload vs rolling restart detection, and native X.509 certificate lifecycle for encrypted cluster communication.
      </p>
    </div>

    <!-- Navigation Tabs -->
    <div class="tab-controls">
      <button
        class="tab-btn {activeTab === 'config' ? 'active' : ''}"
        onclick={() => (activeTab = 'config')}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/>
        </svg>
        Config Templates & Parameter Diff
      </button>

      <button
        class="tab-btn {activeTab === 'pki' ? 'active' : ''}"
        onclick={() => (activeTab = 'pki')}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
        mTLS PKI Certificate Authority
        <span class="badge-count">{$pkiCertificates.length}</span>
      </button>
    </div>
  </header>

  <!-- TAB 1: CONFIG TEMPLATES & ROLLING DIFF -->
  {#if activeTab === 'config'}
    <div class="tab-pane">
      <!-- Selector & Stats Banner -->
      <div class="config-toolbar">
        <div class="template-selector-group">
          <label for="template-select">Active Template:</label>
          <select id="template-select" bind:value={selectedTemplateId} class="styled-select">
            {#each $configTemplates as tmpl}
              <option value={tmpl.id}>{tmpl.name} (v{tmpl.version})</option>
            {/each}
          </select>
          <span class="engine-pill {currentTemplate.engine}">{currentTemplate.engine.toUpperCase()}</span>
        </div>

        <div class="toolbar-actions">
          <button class="btn btn-secondary" onclick={startEditTemplate}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="btn-icon">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
            Edit Template Content
          </button>
          <button class="btn btn-primary" onclick={handleRollout}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="btn-icon">
              <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/>
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
            </svg>
            Apply Rolling Configuration
          </button>
        </div>
      </div>

      <!-- Diff & Parameter Analysis Grid -->
      <div class="content-split-grid">
        <!-- Parameter Diff Table -->
        <div class="card diff-card">
          <div class="card-header">
            <div>
              <h3>Parameter Diff vs Active Database</h3>
              <p class="card-desc">Comparing target template values against runtime settings on <strong>{currentTemplate.clusterName}</strong>.</p>
            </div>
            <div class="diff-summary-badges">
              <span class="pill pill-warning">
                {sampleDiffs.filter(d => d.restartRequired).length} Restart Required
              </span>
              <span class="pill pill-success">
                {sampleDiffs.filter(d => !d.restartRequired).length} Dynamic Reload OK
              </span>
            </div>
          </div>

          <div class="table-responsive">
            <table class="data-table">
              <thead>
                <tr>
                  <th>Parameter</th>
                  <th>Current Runtime</th>
                  <th>Target Template</th>
                  <th>Action Required</th>
                  <th>Rationale</th>
                </tr>
              </thead>
              <tbody>
                {#each sampleDiffs as item}
                  <tr>
                    <td class="param-cell">
                      <code>{item.param}</code>
                      <span class="param-unit">{item.unit}</span>
                    </td>
                    <td class="val-current">
                      <code>{item.current}</code>
                    </td>
                    <td class="val-target">
                      <code>{item.target}</code>
                    </td>
                    <td>
                      {#if item.restartRequired}
                        <span class="action-tag tag-restart">
                          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M12 2v4"/><path d="M12 18v4"/><path d="M4.93 4.93l2.83 2.83"/><path d="M16.24 16.24l2.83 2.83"/><path d="M2 12h4"/><path d="M18 12h4"/><path d="M4.93 19.07l2.83-2.83"/><path d="M16.24 7.76l2.83-2.83"/>
                          </svg>
                          Rolling Restart
                        </span>
                      {:else}
                        <span class="action-tag tag-reload">
                          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <polyline points="20 6 9 17 4 12"/>
                          </svg>
                          Dynamic Reload
                        </span>
                      {/if}
                    </td>
                    <td class="rationale-cell">{item.rationale}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>

        <!-- Template Meta Card -->
        <div class="card meta-card">
          <h3>Template Metadata</h3>
          <div class="meta-field">
            <span class="label">Config File Target:</span>
            <span class="value code-val">{currentTemplate.filePath}</span>
          </div>
          <div class="meta-field">
            <span class="label">Target Engine:</span>
            <span class="value">{currentTemplate.engine.toUpperCase()}</span>
          </div>
          <div class="meta-field">
            <span class="label">Template Version:</span>
            <span class="value">v{currentTemplate.version}</span>
          </div>
          <div class="meta-field">
            <span class="label">Last Modified:</span>
            <span class="value">{currentTemplate.updatedAt}</span>
          </div>

          <div class="audit-hint-box">
            <h4>Rolling Restart Safety Protocol</h4>
            <p>
              When applying parameters requiring restart, Nodlance executes a progressive rolling restart across replica nodes first, verifies cluster replication health, then triggers a controlled failover before restarting the primary node.
            </p>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- TAB 2: PKI CERTIFICATE AUTHORITY -->
  {#if activeTab === 'pki'}
    <div class="tab-pane">
      <!-- PKI CA Overview Banner -->
      <div class="pki-hero-card">
        <div class="ca-identity">
          <div class="shield-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10"/>
              <path d="m9 12 2 2 4-4"/>
            </svg>
          </div>
          <div>
            <h2>Nodlance Root Certificate Authority (Internal PKI)</h2>
            <p class="ca-meta">
              X.509 v3 Authority • SHA-256 with RSA-4096 / ECDSA P-256 • Automated mTLS Node & Proxy Enrollment
            </p>
          </div>
        </div>

        <button class="btn btn-primary" onclick={() => (showIssueModal = true)}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="btn-icon">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          Issue Node / Client Certificate
        </button>
      </div>

      <!-- Certificates Table -->
      <div class="card certs-card">
        <div class="card-header">
          <div>
            <h3>Issued Cluster & Node Certificates</h3>
            <p class="card-desc">Active mTLS certificates securing database listeners, replication streams, and proxy connections.</p>
          </div>
          <span class="total-badge">{$pkiCertificates.filter(c => !c.isRevoked).length} Active Certificates</span>
        </div>

        <div class="table-responsive">
          <table class="data-table">
            <thead>
              <tr>
                <th>Common Name (CN)</th>
                <th>Type</th>
                <th>Cluster Scope</th>
                <th>SHA-256 Fingerprint</th>
                <th>Valid Until</th>
                <th>Status</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {#each $pkiCertificates as cert}
                <tr class={cert.isRevoked ? 'revoked-row' : ''}>
                  <td class="cn-cell">
                    <div class="cert-icon-wrapper">
                      {#if cert.certType === 'root_ca'}
                        <span class="type-icon ca">CA</span>
                      {:else if cert.certType === 'server'}
                        <span class="type-icon server">SRV</span>
                      {:else}
                        <span class="type-icon client">CLI</span>
                      {/if}
                    </div>
                    <div>
                      <strong>{cert.commonName}</strong>
                    </div>
                  </td>
                  <td>
                    <span class="cert-type-pill {cert.certType}">
                      {cert.certType.replace('_', ' ').toUpperCase()}
                    </span>
                  </td>
                  <td>{cert.clusterName}</td>
                  <td>
                    <code class="fingerprint-snippet" title={cert.fingerprintSha256}>
                      {cert.fingerprintSha256.substring(0, 16)}...
                    </code>
                  </td>
                  <td>
                    <div>{cert.expiresAt}</div>
                    {#if !cert.isRevoked}
                      <small class="days-pill {cert.daysLeft < 30 ? 'expiring' : 'healthy'}">
                        {cert.daysLeft} days remaining
                      </small>
                    {/if}
                  </td>
                  <td>
                    {#if cert.isRevoked}
                      <span class="status-pill status-revoked">REVOKED</span>
                    {:else}
                      <span class="status-pill status-active">ACTIVE</span>
                    {/if}
                  </td>
                  <td class="actions-cell">
                    <button class="btn-icon-subtle" onclick={() => downloadCertPem(cert)} title="Download PEM Certificate">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/>
                      </svg>
                    </button>
                    {#if !cert.isRevoked && cert.certType !== 'root_ca'}
                      <button class="btn-icon-subtle danger" onclick={() => handleRevoke(cert)} title="Revoke Certificate">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/>
                        </svg>
                      </button>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- Modal: Edit Template Content -->
{#if isEditingTemplate}
  <div class="modal-backdrop">
    <div class="modal-card wide">
      <div class="modal-header">
        <h3>Edit Template: {currentTemplate.name}</h3>
        <button class="modal-close" onclick={() => (isEditingTemplate = false)}>×</button>
      </div>
      <div class="modal-body">
        <label for="template-editor" class="editor-label">Configuration Syntax ({currentTemplate.filePath}):</label>
        <textarea
          id="template-editor"
          class="template-textarea"
          rows="18"
          bind:value={editableContent}
        ></textarea>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={() => (isEditingTemplate = false)}>Cancel</button>
        <button class="btn btn-primary" onclick={saveTemplate}>Save & Bump Version</button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Issue New Certificate -->
{#if showIssueModal}
  <div class="modal-backdrop">
    <div class="modal-card">
      <div class="modal-header">
        <h3>Issue New X.509 Certificate</h3>
        <button class="modal-close" onclick={() => (showIssueModal = false)}>×</button>
      </div>
      <form onsubmit={handleIssueCert}>
        <div class="modal-body form-grid">
          <div class="form-group">
            <label for="cert-cluster">Target Database Cluster</label>
            <select id="cert-cluster" bind:value={newCertClusterId} class="styled-select">
              {#each $clusters as c}
                <option value={c.id}>{c.name} ({c.engine})</option>
              {/each}
            </select>
          </div>

          <div class="form-group">
            <label for="cert-cn">Common Name (CN / FQDN)</label>
            <input
              id="cert-cn"
              type="text"
              class="styled-input"
              placeholder="e.g. db-node-04.prod.internal"
              bind:value={newCertCn}
              required
            />
          </div>

          <div class="form-group">
            <label for="cert-type">Certificate Usage Type</label>
            <select id="cert-type" bind:value={newCertType} class="styled-select">
              <option value="server">Server / Database Node</option>
              <option value="client">Client App / Connection Pooler</option>
            </select>
          </div>

          <div class="form-group">
            <label for="cert-validity">Validity Period</label>
            <select id="cert-validity" bind:value={newCertValidity} class="styled-select">
              <option value={90}>90 Days (Recommended)</option>
              <option value={365}>1 Year (365 Days)</option>
              <option value={730}>2 Years (730 Days)</option>
            </select>
          </div>
        </div>
        <div class="modal-footer">
          <button type="button" class="btn btn-secondary" onclick={() => (showIssueModal = false)}>Cancel</button>
          <button type="submit" class="btn btn-primary">Generate & Sign Certificate</button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .config-pki-container {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 24px;
  }

  .view-header {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .title-with-pill {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .title-with-pill h1 {
    margin: 0;
    font-size: 1.6rem;
    font-weight: 700;
  }

  .engine-badge {
    padding: 4px 10px;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
  }

  .engine-badge.mTLS {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.3);
  }

  .engine-badge.diff {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
    border: 1px solid rgba(59, 130, 246, 0.3);
  }

  .subtitle {
    margin: 0;
    color: var(--text-muted, #94a3b8);
    font-size: 0.95rem;
    max-width: 900px;
  }

  .tab-controls {
    display: flex;
    gap: 8px;
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
    padding-bottom: 4px;
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 18px;
    border: none;
    background: transparent;
    color: var(--text-muted, #94a3b8);
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: all 0.2s ease;
  }

  .tab-btn svg {
    width: 18px;
    height: 18px;
  }

  .tab-btn:hover {
    color: var(--text-main, #f8fafc);
  }

  .tab-btn.active {
    color: var(--color-primary, #6366f1);
    border-bottom-color: var(--color-primary, #6366f1);
  }

  .badge-count {
    background: rgba(99, 102, 241, 0.2);
    color: #818cf8;
    padding: 2px 7px;
    border-radius: 12px;
    font-size: 0.75rem;
  }

  /* Toolbar */
  .config-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--bg-surface, #1e293b);
    padding: 16px 20px;
    border-radius: 12px;
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
    flex-wrap: wrap;
    gap: 16px;
  }

  .template-selector-group {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .template-selector-group label {
    font-weight: 600;
    font-size: 0.9rem;
  }

  .styled-select, .styled-input {
    background: var(--bg-input, #0f172a);
    color: var(--text-main, #f8fafc);
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.15));
    border-radius: 6px;
    padding: 8px 12px;
    font-size: 0.9rem;
  }

  .engine-pill {
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 700;
    background: rgba(255, 255, 255, 0.1);
  }

  .engine-pill.postgres { background: rgba(59, 130, 246, 0.2); color: #60a5fa; }
  .engine-pill.mysql { background: rgba(245, 158, 11, 0.2); color: #fbbf24; }
  .engine-pill.galera { background: rgba(168, 85, 247, 0.2); color: #c084fc; }

  .toolbar-actions {
    display: flex;
    gap: 10px;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.88rem;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .btn-icon {
    width: 16px;
    height: 16px;
  }

  .btn-primary {
    background: var(--color-primary, #6366f1);
    color: white;
  }

  .btn-primary:hover {
    background: #4f46e5;
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-main, #f8fafc);
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.15));
  }

  .btn-secondary:hover {
    background: rgba(255, 255, 255, 0.14);
  }

  /* Grid */
  .content-split-grid {
    display: grid;
    grid-template-columns: 2.4fr 1fr;
    gap: 20px;
    margin-top: 20px;
  }

  @media (max-width: 1024px) {
    .content-split-grid {
      grid-template-columns: 1fr;
    }
  }

  .card {
    background: var(--bg-surface, #1e293b);
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
    border-radius: 12px;
    padding: 20px;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 16px;
  }

  .card-header h3 {
    margin: 0 0 4px 0;
    font-size: 1.15rem;
  }

  .card-desc {
    margin: 0;
    color: var(--text-muted, #94a3b8);
    font-size: 0.85rem;
  }

  .diff-summary-badges {
    display: flex;
    gap: 8px;
  }

  .pill {
    padding: 4px 10px;
    border-radius: 20px;
    font-size: 0.78rem;
    font-weight: 700;
  }

  .pill-warning {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .pill-success {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.3);
  }

  /* Tables */
  .table-responsive {
    overflow-x: auto;
  }

  .data-table {
    width: 100%;
    border-collapse: collapse;
    text-align: left;
    font-size: 0.88rem;
  }

  .data-table th {
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
    color: var(--text-muted, #94a3b8);
    font-weight: 600;
  }

  .data-table td {
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.05));
    vertical-align: middle;
  }

  .param-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .param-unit {
    font-size: 0.72rem;
    color: var(--text-muted, #94a3b8);
  }

  .val-current code {
    color: #f87171;
  }

  .val-target code {
    color: #34d399;
    font-weight: 600;
  }

  .action-tag {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .action-tag svg {
    width: 14px;
    height: 14px;
  }

  .tag-restart {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
  }

  .tag-reload {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .rationale-cell {
    color: var(--text-muted, #94a3b8);
    font-size: 0.82rem;
  }

  /* Meta Card */
  .meta-field {
    display: flex;
    justify-content: space-between;
    padding: 10px 0;
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.05));
    font-size: 0.88rem;
  }

  .meta-field .label {
    color: var(--text-muted, #94a3b8);
  }

  .meta-field .value {
    font-weight: 600;
  }

  .code-val {
    font-family: monospace;
    font-size: 0.8rem;
  }

  .audit-hint-box {
    margin-top: 24px;
    background: rgba(99, 102, 241, 0.08);
    border: 1px solid rgba(99, 102, 241, 0.2);
    border-radius: 8px;
    padding: 14px;
  }

  .audit-hint-box h4 {
    margin: 0 0 6px 0;
    font-size: 0.88rem;
    color: #a5b4fc;
  }

  .audit-hint-box p {
    margin: 0;
    font-size: 0.8rem;
    color: var(--text-muted, #94a3b8);
    line-height: 1.4;
  }

  /* PKI Tab */
  .pki-hero-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: linear-gradient(135deg, rgba(30, 41, 59, 0.95), rgba(15, 23, 42, 0.95));
    border: 1px solid rgba(99, 102, 241, 0.25);
    border-radius: 12px;
    padding: 24px;
    flex-wrap: wrap;
    gap: 16px;
    margin-bottom: 24px;
  }

  .ca-identity {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .shield-icon {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .shield-icon svg {
    width: 28px;
    height: 28px;
  }

  .ca-identity h2 {
    margin: 0 0 4px 0;
    font-size: 1.25rem;
  }

  .ca-meta {
    margin: 0;
    color: var(--text-muted, #94a3b8);
    font-size: 0.85rem;
  }

  .cn-cell {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .cert-icon-wrapper .type-icon {
    font-size: 0.65rem;
    font-weight: 800;
    padding: 3px 6px;
    border-radius: 4px;
  }

  .type-icon.ca { background: rgba(168, 85, 247, 0.25); color: #c084fc; }
  .type-icon.server { background: rgba(59, 130, 246, 0.25); color: #60a5fa; }
  .type-icon.client { background: rgba(16, 185, 129, 0.25); color: #34d399; }

  .cert-type-pill {
    padding: 2px 7px;
    border-radius: 4px;
    font-size: 0.72rem;
    font-weight: 700;
  }

  .cert-type-pill.root_ca { background: rgba(168, 85, 247, 0.2); color: #c084fc; }
  .cert-type-pill.server { background: rgba(59, 130, 246, 0.2); color: #60a5fa; }
  .cert-type-pill.client { background: rgba(16, 185, 129, 0.2); color: #34d399; }

  .fingerprint-snippet {
    background: rgba(0, 0, 0, 0.3);
    padding: 3px 6px;
    border-radius: 4px;
    font-size: 0.75rem;
  }

  .days-pill {
    display: inline-block;
    font-size: 0.72rem;
    font-weight: 600;
  }

  .days-pill.healthy { color: #34d399; }
  .days-pill.expiring { color: #f87171; }

  .status-pill {
    padding: 3px 8px;
    border-radius: 12px;
    font-size: 0.72rem;
    font-weight: 700;
  }

  .status-active {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
  }

  .status-revoked {
    background: rgba(239, 68, 68, 0.2);
    color: #f87171;
  }

  .revoked-row {
    opacity: 0.6;
    background: rgba(239, 68, 68, 0.04);
  }

  .actions-cell {
    display: flex;
    gap: 8px;
  }

  .btn-icon-subtle {
    background: transparent;
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
    border-radius: 6px;
    color: var(--text-muted, #94a3b8);
    cursor: pointer;
    padding: 6px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .btn-icon-subtle svg {
    width: 16px;
    height: 16px;
  }

  .btn-icon-subtle:hover {
    color: white;
    background: rgba(255, 255, 255, 0.1);
  }

  .btn-icon-subtle.danger:hover {
    color: #f87171;
    border-color: #f87171;
    background: rgba(239, 68, 68, 0.15);
  }

  /* Modals */
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 20px;
  }

  .modal-card {
    background: var(--bg-surface, #1e293b);
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.15));
    border-radius: 12px;
    width: 100%;
    max-width: 500px;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5);
  }

  .modal-card.wide {
    max-width: 800px;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1.1rem;
  }

  .modal-close {
    background: transparent;
    border: none;
    font-size: 1.5rem;
    color: var(--text-muted, #94a3b8);
    cursor: pointer;
  }

  .modal-body {
    padding: 20px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 20px;
    border-top: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
  }

  .form-grid {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .form-group label {
    font-size: 0.85rem;
    font-weight: 600;
  }

  .editor-label {
    display: block;
    margin-bottom: 8px;
    font-size: 0.85rem;
    color: var(--text-muted, #94a3b8);
  }

  .template-textarea {
    width: 100%;
    background: #0b1120;
    color: #e2e8f0;
    font-family: monospace;
    font-size: 0.85rem;
    padding: 12px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-sizing: border-box;
    resize: vertical;
  }
</style>
