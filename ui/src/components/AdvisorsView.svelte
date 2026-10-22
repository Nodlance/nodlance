<script>
  import {
    advisorRules,
    advisorFindings,
    slowQueries,
    activeClusterId,
    clusters,
    runAdvisorScan,
    resolveAdvisorFinding,
    toggleAdvisorRule,
    openModal,
    showToast
  } from '../lib/stores.js';

  let activeTab = $state('findings'); // 'findings' | 'queries' | 'rules'
  let selectedCategory = $state('all');

  function handleScan() {
    runAdvisorScan($activeClusterId);
  }

  let filteredFindings = $derived(
    $advisorFindings.filter(f => selectedCategory === 'all' || f.ruleId.includes(selectedCategory))
  );

  function getSeverityClass(sev) {
    switch (sev?.toLowerCase()) {
      case 'critical': return 'badge-critical';
      case 'warning': return 'badge-warning';
      default: return 'badge-info';
    }
  }

  function getCategoryClass(cat) {
    switch (cat?.toLowerCase()) {
      case 'performance': return 'cat-perf';
      case 'capacity': return 'cat-cap';
      case 'security': return 'cat-sec';
      case 'availability': return 'cat-avail';
      default: return 'cat-perf';
    }
  }
</script>

<section class="view-panel active" id="viewAdvisors">
  <div class="view-header">
    <div>
      <h1 class="view-title">DBA Performance Advisors & Query Digest</h1>
      <p class="view-subtitle">Embedded Rhai scriptable heuristic rules evaluating buffer pools, table bloats, connection limits, and live query telemetry.</p>
    </div>
    <div class="view-actions">
      <button class="btn btn-secondary btn-sm" onclick={() => showToast('Opening Rhai script editor...', 'info')}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
        <span>Custom Rhai Rule</span>
      </button>
      <button class="btn btn-primary btn-sm" onclick={handleScan}>
        <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="2" x2="22" y1="12" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
        <span>Run Advisor Scan</span>
      </button>
    </div>
  </div>

  <!-- Advisor Navigation Tabs -->
  <div class="advisor-tabs">
    <button class="tab-btn {activeTab === 'findings' ? 'active' : ''}" onclick={() => activeTab = 'findings'}>
      Active Findings
      {#if $advisorFindings.length > 0}
        <span class="tab-badge">{$advisorFindings.length}</span>
      {/if}
    </button>
    <button class="tab-btn {activeTab === 'queries' ? 'active' : ''}" onclick={() => activeTab = 'queries'}>
      Slow Query Digest
      <span class="tab-badge query-badge">Live</span>
    </button>
    <button class="tab-btn {activeTab === 'rules' ? 'active' : ''}" onclick={() => activeTab = 'rules'}>
      Rhai Heuristic Rulebook ({$advisorRules.length})
    </button>
  </div>

  {#if activeTab === 'findings'}
    <!-- Tab 1: Active Findings -->
    <div class="card table-card">
      <div class="card-header" style="justify-content: space-between;">
        <div>
          <h2 class="card-title">DBA Recommendations & Anomaly Findings</h2>
          <p class="card-desc">Real-time heuristics flagged by sandboxed Rhai execution over cluster telemetry.</p>
        </div>
        <div class="filter-pills">
          <button class="pill {selectedCategory === 'all' ? 'active' : ''}" onclick={() => selectedCategory = 'all'}>All</button>
          <button class="pill {selectedCategory === 'performance' ? 'active' : ''}" onclick={() => selectedCategory = 'performance'}>Performance</button>
          <button class="pill {selectedCategory === 'capacity' ? 'active' : ''}" onclick={() => selectedCategory = 'capacity'}>Capacity</button>
          <button class="pill {selectedCategory === 'security' ? 'active' : ''}" onclick={() => selectedCategory = 'security'}>Security</button>
        </div>
      </div>

      <div class="table-responsive">
        <table class="data-table">
          <thead>
            <tr>
              <th>Severity</th>
              <th>Cluster</th>
              <th>Advisor Rule</th>
              <th>Observed Anomaly</th>
              <th>Actionable DBA Recommendation</th>
              <th>Detected</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            {#if filteredFindings.length === 0}
              <tr>
                <td colspan="7" style="text-align: center; padding: 32px; color: var(--text-muted);">
                  No active anomalies detected! All heuristic checks are passing.
                </td>
              </tr>
            {/if}
            {#each filteredFindings as f (f.id)}
              <tr>
                <td>
                  <span class="severity-badge {getSeverityClass(f.severity)}">
                    {f.severity.toUpperCase()}
                  </span>
                </td>
                <td style="font-weight: 600;">{f.clusterName}</td>
                <td style="font-size: 12px; font-weight: 600;">{f.ruleName}</td>
                <td style="color: var(--text-main); font-size: 12px; max-width: 260px;">{f.summary}</td>
                <td style="color: var(--text-muted); font-size: 12px; max-width: 320px;">
                  <span class="recommendation-box">{f.recommendation}</span>
                </td>
                <td style="color: var(--text-muted); font-size: 11px;">{f.detectedAt}</td>
                <td>
                  <button
                    class="btn btn-secondary btn-sm"
                    style="padding: 2px 8px; font-size: 11px;"
                    onclick={() => resolveAdvisorFinding(f.id)}
                  >
                    Resolve
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>

  {:else if activeTab === 'queries'}
    <!-- Tab 2: Slow Query Digest -->
    <div class="card table-card">
      <div class="card-header">
        <div>
          <h2 class="card-title">Normalized Slow Query Digest Telemetry</h2>
          <p class="card-desc">Aggregated statement summaries from pg_stat_statements and performance_schema with latency percentiles.</p>
        </div>
      </div>

      <div class="table-responsive">
        <table class="data-table">
          <thead>
            <tr>
              <th>Query Digest (Normalized SQL)</th>
              <th>Cluster / DB</th>
              <th>Exec Count</th>
              <th>Mean Latency</th>
              <th>P95 Latency</th>
              <th>Max Latency</th>
              <th>Rows Examined / Sent</th>
              <th>Last Seen</th>
            </tr>
          </thead>
          <tbody>
            {#each $slowQueries as q (q.queryHash)}
              <tr>
                <td style="max-width: 360px;">
                  <div class="query-code font-mono">{q.digestText}</div>
                </td>
                <td>
                  <div style="font-weight: 600; font-size: 12px;">{q.clusterName}</div>
                  <div class="badge badge-neutral font-mono" style="font-size: 10px; margin-top: 2px;">{q.databaseName}</div>
                </td>
                <td class="font-mono">{q.executionCount.toLocaleString()}</td>
                <td>
                  <span class="latency-pill {q.meanLatencyMs > 500 ? 'text-rose' : 'text-amber'}">
                    {q.meanLatencyMs} ms
                  </span>
                </td>
                <td class="font-mono" style="font-size: 11px;">{q.p95LatencyMs} ms</td>
                <td class="font-mono text-rose" style="font-size: 11px; font-weight: 600;">{q.maxLatencyMs} ms</td>
                <td>
                  <span class="font-mono" style="font-size: 11px;">
                    {(q.rowsExamined / 1000000).toFixed(1)}M / {(q.rowsSent / 1000).toFixed(0)}k
                  </span>
                </td>
                <td style="color: var(--text-muted); font-size: 11px;">{q.lastSeen}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>

  {:else if activeTab === 'rules'}
    <!-- Tab 3: Rhai Heuristic Rulebook -->
    <div class="card table-card">
      <div class="card-header">
        <div>
          <h2 class="card-title">Embedded Rhai Heuristic Rules Catalogue</h2>
          <p class="card-desc">Production DBA heuristics evaluated automatically in a sandboxed, zero-overhead Rust environment.</p>
        </div>
      </div>

      <div class="table-responsive">
        <table class="data-table">
          <thead>
            <tr>
              <th>Rule ID & Name</th>
              <th>Engine</th>
              <th>Category</th>
              <th>Severity</th>
              <th>Rhai Script Predicate</th>
              <th>Status</th>
              <th>Toggle</th>
            </tr>
          </thead>
          <tbody>
            {#each $advisorRules as r (r.id)}
              <tr>
                <td>
                  <div style="font-weight: 600; font-size: 13px;">{r.name}</div>
                  <div class="font-mono" style="font-size: 11px; color: var(--text-muted);">{r.id}</div>
                </td>
                <td><span class="badge badge-neutral">{r.engine.toUpperCase()}</span></td>
                <td>
                  <span class="category-badge {getCategoryClass(r.category)}">{r.category}</span>
                </td>
                <td>
                  <span class="severity-badge {getSeverityClass(r.severity)}">
                    {r.severity.toUpperCase()}
                  </span>
                </td>
                <td>
                  <code class="script-snippet font-mono">{r.scriptContent}</code>
                </td>
                <td>
                  {#if r.isEnabled}
                    <span class="status-indicator text-emerald">ACTIVE</span>
                  {:else}
                    <span class="status-indicator" style="color: var(--text-muted);">DISABLED</span>
                  {/if}
                </td>
                <td>
                  <button
                    class="btn btn-secondary btn-sm"
                    style="padding: 2px 8px; font-size: 11px;"
                    onclick={() => toggleAdvisorRule(r.id)}
                  >
                    {r.isEnabled ? 'Disable' : 'Enable'}
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</section>

<style>
  .advisor-tabs {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
    padding-bottom: 8px;
  }
  .tab-btn {
    background: transparent;
    border: none;
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 6px;
    display: flex;
    align-items: center;
    gap: 8px;
    transition: all 0.2s ease;
  }
  .tab-btn:hover {
    color: var(--text-main);
    background: rgba(255, 255, 255, 0.04);
  }
  .tab-btn.active {
    color: var(--primary, #3b82f6);
    background: rgba(59, 130, 246, 0.1);
  }
  .tab-badge {
    padding: 1px 6px;
    font-size: 10px;
    border-radius: 9999px;
    background: #ef4444;
    color: white;
  }
  .query-badge {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.4);
  }
  .filter-pills {
    display: flex;
    gap: 6px;
  }
  .pill {
    background: transparent;
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
    padding: 3px 10px;
    font-size: 11px;
    border-radius: 9999px;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.2s;
  }
  .pill.active {
    background: var(--primary, #3b82f6);
    color: white;
    border-color: var(--primary, #3b82f6);
  }
  .severity-badge {
    display: inline-block;
    padding: 2px 6px;
    font-size: 10px;
    font-weight: 700;
    border-radius: 4px;
    letter-spacing: 0.5px;
  }
  .badge-critical {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }
  .badge-warning {
    background: rgba(245, 158, 11, 0.15);
    color: #f59e0b;
    border: 1px solid rgba(245, 158, 11, 0.3);
  }
  .badge-info {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
    border: 1px solid rgba(59, 130, 246, 0.3);
  }
  .category-badge {
    display: inline-block;
    padding: 2px 6px;
    font-size: 10px;
    font-weight: 600;
    text-transform: capitalize;
    border-radius: 4px;
  }
  .cat-perf { background: rgba(168, 85, 247, 0.15); color: #c084fc; }
  .cat-cap { background: rgba(249, 115, 22, 0.15); color: #fb923c; }
  .cat-sec { background: rgba(239, 68, 68, 0.15); color: #f87171; }
  .cat-avail { background: rgba(16, 185, 129, 0.15); color: #34d399; }
  .recommendation-box {
    display: block;
    background: var(--bg-surface-secondary, rgba(0, 0, 0, 0.04));
    padding: 4px 8px;
    border-radius: 4px;
    border-left: 2px solid var(--primary, #3b82f6);
  }
  .query-code {
    font-size: 11px;
    background: var(--bg-surface-secondary, rgba(0, 0, 0, 0.05));
    padding: 4px 6px;
    border-radius: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .script-snippet {
    background: rgba(0, 0, 0, 0.06);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 11px;
    color: #a855f7;
  }
  .latency-pill {
    font-family: monospace;
    font-size: 12px;
    font-weight: 700;
  }
</style>
