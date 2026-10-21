/**
 * Nodlance ClusterControl MVP
 * Frontend State Machine, Live Topology Graph, Guarded Provisioning & HA Orchestrator
 */

(function () {
  'use strict';

  // --- Initial Seed Data (Severalnines Enterprise Grade Fleet) ---
  const DEMO_CLUSTERS = [
    {
      id: 'c1a7001-pg16-prod',
      name: 'prod-pg16-ha-primary',
      engine: 'postgres',
      engineName: 'PostgreSQL 16.4',
      provider: 'Linux Baremetal',
      status: 'healthy',
      qps: 12450,
      avgLagMs: 8,
      storageGb: 840,
      nodesCount: 3,
      quorum: '3/3 Votes (Healthy)',
      epoch: 4,
      term: 12,
      nodes: [
        {
          id: 'node-pg-01',
          host: 'db-pg-node-01.prod.internal',
          ip: '192.168.10.21',
          port: 5432,
          role: 'primary',
          state: 'healthy',
          lagMs: 0,
          lagBytes: 0,
          lsn: '0/1F8A9020',
          cpu: '24%',
          mem: '62%',
          uptime: '42d 11h',
          cacheHitRatio: '99.4%'
        },
        {
          id: 'node-pg-02',
          host: 'db-pg-node-02.prod.internal',
          ip: '192.168.10.22',
          port: 5432,
          role: 'sync',
          state: 'healthy',
          lagMs: 6,
          lagBytes: 128,
          lsn: '0/1F8A8FA0',
          cpu: '18%',
          mem: '58%',
          uptime: '42d 11h',
          cacheHitRatio: '99.1%'
        },
        {
          id: 'node-pg-03',
          host: 'db-pg-node-03.prod.internal',
          ip: '192.168.10.23',
          port: 5432,
          role: 'async',
          state: 'healthy',
          lagMs: 14,
          lagBytes: 512,
          lsn: '0/1F8A8E20',
          cpu: '16%',
          mem: '55%',
          uptime: '42d 11h',
          cacheHitRatio: '98.9%'
        }
      ]
    },
    {
      id: 'c2b8002-my84-core',
      name: 'core-mysql-ecommerce',
      engine: 'mysql',
      engineName: 'MySQL 8.4 LTS',
      provider: 'Docker Engine',
      status: 'healthy',
      qps: 8920,
      avgLagMs: 18,
      storageGb: 620,
      nodesCount: 3,
      quorum: '3/3 Members',
      epoch: 2,
      term: 6,
      nodes: [
        {
          id: 'node-my-01',
          host: 'core-mysql-leader.dc1',
          ip: '172.28.0.11',
          port: 3306,
          role: 'primary',
          state: 'healthy',
          lagMs: 0,
          lagBytes: 0,
          lsn: 'GTID 3e1a:1-94820',
          cpu: '31%',
          mem: '70%',
          uptime: '18d 04h',
          cacheHitRatio: '98.7%'
        },
        {
          id: 'node-my-02',
          host: 'core-mysql-replica-1.dc1',
          ip: '172.28.0.12',
          port: 3306,
          role: 'sync',
          state: 'healthy',
          lagMs: 12,
          lagBytes: 256,
          lsn: 'GTID 3e1a:1-94818',
          cpu: '22%',
          mem: '65%',
          uptime: '18d 04h',
          cacheHitRatio: '98.2%'
        },
        {
          id: 'node-my-03',
          host: 'core-mysql-replica-2.dc1',
          ip: '172.28.0.13',
          port: 3306,
          role: 'async',
          state: 'healthy',
          lagMs: 24,
          lagBytes: 1024,
          lsn: 'GTID 3e1a:1-94809',
          cpu: '19%',
          mem: '61%',
          uptime: '18d 04h',
          cacheHitRatio: '97.9%'
        }
      ]
    },
    {
      id: 'c3c9003-ma11-analytics',
      name: 'bi-reporting-galera',
      engine: 'mariadb',
      engineName: 'MariaDB 11.4 Galera',
      provider: 'Kubernetes',
      status: 'degraded',
      qps: 2150,
      avgLagMs: 48,
      storageGb: 380,
      nodesCount: 2,
      quorum: '2/3 Members (Degraded)',
      epoch: 7,
      term: 19,
      nodes: [
        {
          id: 'node-ma-01',
          host: 'galera-node-0.k8s.cluster',
          ip: '10.244.2.45',
          port: 3306,
          role: 'primary',
          state: 'healthy',
          lagMs: 0,
          lagBytes: 0,
          lsn: 'WSREP seqno 109240',
          cpu: '45%',
          mem: '82%',
          uptime: '8d 14h',
          cacheHitRatio: '96.5%'
        },
        {
          id: 'node-ma-02',
          host: 'galera-node-1.k8s.cluster',
          ip: '10.244.3.89',
          port: 3306,
          role: 'async',
          state: 'healthy',
          lagMs: 48,
          lagBytes: 4096,
          lsn: 'WSREP seqno 109232',
          cpu: '38%',
          mem: '76%',
          uptime: '8d 14h',
          cacheHitRatio: '95.8%'
        }
      ]
    }
  ];

  const DEMO_HOSTS = [
    {
      name: 'db-pg-node-01.prod.internal',
      ip: '192.168.10.21',
      os: 'Ubuntu 24.04 LTS (x86_64)',
      agentStatus: 'Enrolled & Active',
      heartbeatSec: 1,
      cpu: 24,
      ram: 62
    },
    {
      name: 'db-pg-node-02.prod.internal',
      ip: '192.168.10.22',
      os: 'Ubuntu 24.04 LTS (x86_64)',
      agentStatus: 'Enrolled & Active',
      heartbeatSec: 2,
      cpu: 18,
      ram: 58
    },
    {
      name: 'db-pg-node-03.prod.internal',
      ip: '192.168.10.23',
      os: 'Ubuntu 24.04 LTS (x86_64)',
      agentStatus: 'Enrolled & Active',
      heartbeatSec: 1,
      cpu: 16,
      ram: 55
    },
    {
      name: 'docker-host-01.infra',
      ip: '172.28.0.10',
      os: 'Debian 12 Bookworm',
      agentStatus: 'Enrolled & Active',
      heartbeatSec: 3,
      cpu: 31,
      ram: 70
    },
    {
      name: 'k8s-worker-pool-02',
      ip: '10.244.0.12',
      os: 'Rocky Linux 9.4',
      agentStatus: 'Enrolled & Active',
      heartbeatSec: 1,
      cpu: 45,
      ram: 82
    }
  ];

  const DEMO_BACKUPS = [
    {
      id: 'bk_pg_98a71b4c',
      cluster: 'prod-pg16-ha-primary',
      type: 'Full Physical Snapshot',
      engine: 'PostgreSQL 16',
      size: '42.8 GB',
      checksum: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
      status: 'VERIFIED',
      completedAt: 'Today, 04:00 AM'
    },
    {
      id: 'bk_my_77d20e11',
      cluster: 'core-mysql-ecommerce',
      type: 'XtraBackup Physical',
      engine: 'MySQL 8.4',
      size: '31.2 GB',
      checksum: 'ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb',
      status: 'VERIFIED',
      completedAt: 'Yesterday, 04:00 AM'
    },
    {
      id: 'bk_ma_55f19c32',
      cluster: 'bi-reporting-galera',
      type: 'Mariabackup Snapshot',
      engine: 'MariaDB 11.4',
      size: '18.4 GB',
      checksum: '4e1243bd22c66e76c2ba9eddc1f91394e5b6f56528132ac2abc7452a8ceece47',
      status: 'VERIFIED',
      completedAt: '2 days ago'
    }
  ];

  const DEMO_DRILLS = [
    {
      id: 'drill_res_4812a',
      sourceBackup: 'bk_pg_98a71b4c',
      targetSandbox: 'sandbox-pg-drill-01.tmp',
      duration: '42s',
      corruption: '0 blocks',
      result: 'PASSED (100% Verified)',
      completedAt: 'Today, 04:15 AM'
    },
    {
      id: 'drill_res_3391b',
      sourceBackup: 'bk_my_77d20e11',
      targetSandbox: 'sandbox-my-drill-02.tmp',
      duration: '38s',
      corruption: '0 blocks',
      result: 'PASSED (100% Verified)',
      completedAt: 'Yesterday, 04:14 AM'
    }
  ];

  const DEMO_DISCOVERY = [
    {
      id: 'disc_workload_910',
      host: '192.168.10.45',
      port: 5432,
      engine: 'PostgreSQL',
      version: '15.6',
      role: 'Standalone Instance',
      discoveredAt: '12 mins ago'
    },
    {
      id: 'disc_workload_911',
      host: '172.28.0.55',
      port: 3306,
      engine: 'MySQL',
      version: '8.0.36',
      role: 'Source / Primary',
      discoveredAt: '35 mins ago'
    }
  ];

  const DEMO_ALERTS = [
    {
      id: 'alt_9011',
      severity: 'WARNING',
      target: 'bi-reporting-galera / node-ma-02',
      rule: 'Replication Lag Threshold Exceeded (> 40ms)',
      message: 'Node lag increased to 48ms during batch OLAP processing',
      triggeredAt: '18 mins ago',
      status: 'Active'
    }
  ];

  const DEMO_AUDIT = [
    {
      id: 'aud_8801',
      time: '14:22:10 UTC',
      actor: 'admin (Superadmin)',
      action: 'CLUSTER_EPOCH_ADVANCED',
      target: 'prod-pg16-ha-primary (Epoch #4)',
      status: 'SUCCESS',
      ip: '10.0.4.12'
    },
    {
      id: 'aud_8802',
      time: '13:05:44 UTC',
      actor: 'system::recovery_engine',
      action: 'QUORUM_HEALTH_EVALUATION',
      target: 'core-mysql-ecommerce (3/3 votes)',
      status: 'SUCCESS',
      ip: '127.0.0.1'
    },
    {
      id: 'aud_8803',
      time: '04:15:02 UTC',
      actor: 'nodlance-worker-daemon',
      action: 'RESTORE_DRILL_VALIDATION',
      target: 'bk_pg_98a71b4c',
      status: 'SUCCESS',
      ip: '127.0.0.1'
    }
  ];

  const DEMO_FENCING = [
    {
      id: 'fnc_7741',
      target: 'db-pg-node-old.prod (192.168.10.99)',
      mechanism: 'STONITH (Power Off via IPMI)',
      reason: 'Split-Brain Prevention during failover #3',
      actor: 'admin (Superadmin)',
      status: 'VERIFIED ISOLATED',
      timestamp: 'Yesterday, 18:22'
    }
  ];

  const DEMO_PLANS = [
    {
      id: 'plan_pg_01',
      cluster: 'prod-pg16-analytics',
      provider: 'Linux Baremetal',
      engine: 'PostgreSQL 16 HA',
      steps: '5 / 5',
      status: 'COMPLETED (Guarded)',
      created: 'Today, 10:30 AM'
    }
  ];

  // --- Application State ---
  const state = {
    clusters: JSON.parse(JSON.stringify(DEMO_CLUSTERS)),
    activeClusterId: 'c1a7001-pg16-prod',
    activeView: 'fleet',
    engineFilter: 'all',
    isDemoMode: true,
    hosts: JSON.parse(JSON.stringify(DEMO_HOSTS)),
    backups: JSON.parse(JSON.stringify(DEMO_BACKUPS)),
    drills: JSON.parse(JSON.stringify(DEMO_DRILLS)),
    discovery: JSON.parse(JSON.stringify(DEMO_DISCOVERY)),
    alerts: JSON.parse(JSON.stringify(DEMO_ALERTS)),
    audit: JSON.parse(JSON.stringify(DEMO_AUDIT)),
    fencing: JSON.parse(JSON.stringify(DEMO_FENCING)),
    plans: JSON.parse(JSON.stringify(DEMO_PLANS)),
    wizard: {
      step: 1,
      totalSteps: 5,
      provider: 'linux',
      engine: 'postgres',
      clusterName: 'prod-pg16-analytics',
      nodeCount: 3,
      port: 5432
    }
  };

  // --- DOM Element References ---
  const elements = {
    clusterSelect: document.getElementById('clusterSelect'),
    statClusterCount: document.getElementById('statClusterCount'),
    statNodeCount: document.getElementById('statNodeCount'),
    statAlertCount: document.getElementById('statAlertCount'),
    statAlertDot: document.getElementById('statAlertDot'),
    statQuorumVal: document.getElementById('statQuorumVal'),
    modeToggleBtn: document.getElementById('modeToggleBtn'),
    modeText: document.getElementById('modeText'),
    sidebar: document.getElementById('sidebar'),
    navItems: document.querySelectorAll('.nav-item'),
    viewPanels: document.querySelectorAll('.view-panel'),
    clusterCardsContainer: document.getElementById('clusterCardsContainer'),
    topologySvgWrapper: document.getElementById('topologySvgWrapper'),
    topoClusterName: document.getElementById('topoClusterName'),
    nodesTableBody: document.getElementById('nodesTableBody'),
    candidatesTableBody: document.getElementById('candidatesTableBody'),
    fencingTableBody: document.getElementById('fencingTableBody'),
    plansTableBody: document.getElementById('plansTableBody'),
    backupsTableBody: document.getElementById('backupsTableBody'),
    drillsTableBody: document.getElementById('drillsTableBody'),
    discoveryTableBody: document.getElementById('discoveryTableBody'),
    hostsTableBody: document.getElementById('hostsTableBody'),
    alertsTableBody: document.getElementById('alertsTableBody'),
    auditTableBody: document.getElementById('auditTableBody'),
    toastContainer: document.getElementById('toastContainer'),
    filterBtns: document.querySelectorAll('.filter-btn'),
    // Modals
    deployWizardModal: document.getElementById('deployWizardModal'),
    failoverModal: document.getElementById('failoverModal'),
    fenceModal: document.getElementById('fenceModal'),
    backupModal: document.getElementById('backupModal'),
    scanModal: document.getElementById('scanModal'),
    enrollModal: document.getElementById('enrollModal'),
    // Wizard Controls
    btnStartWizard: document.getElementById('btnStartWizard'),
    btnOpenDeployWizard: document.getElementById('btnOpenDeployWizard'),
    wizBtnNext: document.getElementById('wizBtnNext'),
    wizBtnBack: document.getElementById('wizBtnBack'),
    wizardStepNum: document.getElementById('wizardStepNum'),
    wizardProgressFill: document.getElementById('wizardProgressFill'),
    wizardSteps: document.querySelectorAll('.wizard-step'),
    // Action Buttons
    btnTestFailover: document.getElementById('btnTestFailover'),
    btnRequestPromotion: document.getElementById('btnRequestPromotion'),
    btnOpenDiscovery: document.getElementById('btnOpenDiscovery'),
    btnTriggerScanModal: document.getElementById('btnTriggerScanModal'),
    btnCreateBackupModal: document.getElementById('btnCreateBackupModal'),
    btnTriggerNodeBackup: document.getElementById('btnTriggerNodeBackup'),
    btnManualFenceModal: document.getElementById('btnManualFenceModal'),
    btnEnrollHostModal: document.getElementById('btnEnrollHostModal'),
    btnLaunchFaultInjection: document.getElementById('btnLaunchFaultInjection'),
    btnRefreshFleet: document.getElementById('btnRefreshFleet'),
    btnCopyEnrollCmd: document.getElementById('btnCopyEnrollCmd'),
    btnRegenToken: document.getElementById('btnRegenToken'),
    btnExecutePromotion: document.getElementById('btnExecutePromotion'),
    btnConfirmFence: document.getElementById('btnConfirmFence'),
    btnSubmitBackup: document.getElementById('btnSubmitBackup'),
    btnRunScan: document.getElementById('btnRunScan')
  };

  // --- Helper: Toast Notification ---
  function showToast(message, type = 'info') {
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.innerHTML = `
      <div class="toast-content">${message}</div>
    `;
    elements.toastContainer.appendChild(toast);
    setTimeout(() => {
      toast.style.opacity = '0';
      toast.style.transform = 'translateX(30px)';
      toast.style.transition = 'all 200ms ease';
      setTimeout(() => toast.remove(), 250);
    }, 3800);
  }

  // --- Modal Helpers ---
  function openModal(modal) {
    if (modal) {
      modal.classList.add('open');
      modal.setAttribute('aria-hidden', 'false');
    }
  }

  function closeModal(modal) {
    if (modal) {
      modal.classList.remove('open');
      modal.setAttribute('aria-hidden', 'true');
    }
  }

  // Bind close buttons on all modals
  document.querySelectorAll('[data-close-modal]').forEach(btn => {
    btn.addEventListener('click', (e) => {
      const modal = e.target.closest('.modal-backdrop');
      closeModal(modal);
    });
  });

  // Close modal when clicking on backdrop
  document.querySelectorAll('.modal-backdrop').forEach(modal => {
    modal.addEventListener('click', (e) => {
      if (e.target === modal) {
        closeModal(modal);
      }
    });
  });

  // --- View Navigation Router ---
  function switchView(viewName) {
    state.activeView = viewName;
    elements.navItems.forEach(item => {
      if (item.dataset.view === viewName) {
        item.classList.add('active');
      } else {
        item.classList.remove('active');
      }
    });

    elements.viewPanels.forEach(panel => {
      if (panel.id === `view${viewName.charAt(0).toUpperCase() + viewName.slice(1).replace(/-([a-z])/g, (_, l) => l.toUpperCase())}`) {
        panel.classList.add('active');
      } else {
        panel.classList.remove('active');
      }
    });

    // Special view initializations
    if (viewName === 'topology') {
      renderTopology();
      renderNodesTable();
    } else if (viewName === 'ha-recovery') {
      renderCandidates();
      renderFencingTable();
    } else if (viewName === 'fleet') {
      renderClusterCards();
    }
  }

  elements.navItems.forEach(item => {
    item.addEventListener('click', () => {
      switchView(item.dataset.view);
    });
  });

  // --- Active Cluster Switcher ---
  function populateClusterSelect() {
    elements.clusterSelect.innerHTML = '';
    state.clusters.forEach(c => {
      const opt = document.createElement('option');
      opt.value = c.id;
      opt.textContent = `${c.name} (${c.engine.toUpperCase()})`;
      if (c.id === state.activeClusterId) {
        opt.selected = true;
      }
      elements.clusterSelect.appendChild(opt);
    });
  }

  elements.clusterSelect.addEventListener('change', (e) => {
    state.activeClusterId = e.target.value;
    const c = getActiveCluster();
    if (c) {
      elements.topoClusterName.textContent = c.name;
    }
    renderTopology();
    renderNodesTable();
    renderCandidates();
    showToast(`Switched active cluster context to: ${c ? c.name : ''}`, 'info');
  });

  function getActiveCluster() {
    return state.clusters.find(c => c.id === state.activeClusterId) || state.clusters[0];
  }

  // --- Render Fleet Overview ---
  function renderClusterCards() {
    elements.clusterCardsContainer.innerHTML = '';
    const filtered = state.clusters.filter(c => {
      if (state.engineFilter === 'all') return true;
      return c.engine === state.engineFilter;
    });

    filtered.forEach(c => {
      const card = document.createElement('div');
      card.className = 'cluster-card';
      card.innerHTML = `
        <div class="cluster-card-head">
          <div>
            <span class="cluster-engine-badge ${c.engine === 'postgres' ? 'pg' : c.engine === 'mysql' ? 'my' : 'ma'}">
              ${c.engineName}
            </span>
            <h3 class="cluster-name">${c.name}</h3>
            <span class="cluster-id">${c.id}</span>
          </div>
          <span class="cluster-status-pill ${c.status}">${c.status.toUpperCase()}</span>
        </div>

        <div class="cluster-stats-row">
          <div class="cluster-mini-stat">
            <span class="mini-stat-label">Throughput</span>
            <span class="mini-stat-val">${c.qps.toLocaleString()} QPS</span>
          </div>
          <div class="cluster-mini-stat">
            <span class="mini-stat-label">Replication Lag</span>
            <span class="mini-stat-val text-emerald">${c.avgLagMs} ms</span>
          </div>
          <div class="cluster-mini-stat">
            <span class="mini-stat-label">Nodes</span>
            <span class="mini-stat-val">${c.nodes.length} Active</span>
          </div>
        </div>

        <div class="cluster-card-foot">
          <div class="cluster-provider-tag">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="14" x="2" y="3" rx="2"/></svg>
            <span>${c.provider}</span>
          </div>
          <button class="btn btn-secondary btn-sm" data-switch-cluster="${c.id}">View Topology →</button>
        </div>
      `;

      card.querySelector('[data-switch-cluster]').addEventListener('click', (e) => {
        e.stopPropagation();
        state.activeClusterId = c.id;
        elements.clusterSelect.value = c.id;
        elements.topoClusterName.textContent = c.name;
        switchView('topology');
      });

      card.addEventListener('click', () => {
        state.activeClusterId = c.id;
        elements.clusterSelect.value = c.id;
        elements.topoClusterName.textContent = c.name;
        switchView('topology');
      });

      elements.clusterCardsContainer.appendChild(card);
    });

    // Update global summary pills
    elements.statClusterCount.textContent = state.clusters.length;
    let totalNodes = 0;
    state.clusters.forEach(c => totalNodes += c.nodes.length);
    elements.statNodeCount.textContent = totalNodes;
    elements.statAlertCount.textContent = state.alerts.length;
    document.getElementById('navBadgeAlerts').textContent = state.alerts.length;
  }

  // Filter buttons handler
  elements.filterBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      elements.filterBtns.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      state.engineFilter = btn.dataset.engine;
      renderClusterCards();
    });
  });

  // --- Dynamic SVG Topology Renderer (ClusterControl Signature) ---
  function renderTopology() {
    const cluster = getActiveCluster();
    if (!cluster) return;

    const isDark = document.body.classList.contains('dark-theme');
    const primary = cluster.nodes.find(n => n.role === 'primary') || cluster.nodes[0];
    const standbys = cluster.nodes.filter(n => n.id !== primary.id);

    const svgWidth = 840;
    const svgHeight = 360;
    const primaryX = svgWidth / 2;
    const primaryY = 70;

    const replicaBoxFill = isDark ? '#121d33' : '#ffffff';
    const replicaTextPrimary = isDark ? '#f8fafc' : '#0f172a';
    const replicaTextMuted = isDark ? '#94a3b8' : '#64748b';
    const lagBadgeBg = isDark ? '#0b1220' : '#ffffff';
    const lagBadgeBorder = isDark ? '#1c2b44' : '#e2e8f0';

    let svgHtml = `
      <svg viewBox="0 0 ${svgWidth} ${svgHeight}" width="100%" height="360" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <filter id="glowEffect" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="0" stdDeviation="6" flood-color="#0284c7" flood-opacity="0.3"/>
          </filter>
          <linearGradient id="primaryGrad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#0284c7"/>
            <stop offset="100%" stop-color="#38bdf8"/>
          </linearGradient>
        </defs>
    `;

    // Calculate positions for replicas
    const standbyCount = standbys.length;
    const spacing = svgWidth / (standbyCount + 1);

    standbys.forEach((replica, idx) => {
      const repX = spacing * (idx + 1);
      const repY = 260;

      // Draw connection path from Primary to Replica
      const isSync = replica.role === 'sync';
      const strokeColor = isSync ? '#059669' : '#0284c7';
      const strokeDash = isSync ? 'none' : '5,5';

      svgHtml += `
        <!-- Replication link to ${replica.id} -->
        <path d="M ${primaryX} ${primaryY + 36} C ${primaryX} ${(primaryY + repY) / 2}, ${repX} ${(primaryY + repY) / 2}, ${repX} ${repY - 36}"
              fill="none" stroke="${strokeColor}" stroke-width="2.5" stroke-dasharray="${strokeDash}" opacity="0.85"/>
        
        <!-- Animated streaming packet -->
        <circle r="4" fill="${strokeColor}">
          <animateMotion dur="${isSync ? '1.4s' : '2.2s'}" repeatCount="indefinite"
            path="M ${primaryX} ${primaryY + 36} C ${primaryX} ${(primaryY + repY) / 2}, ${repX} ${(primaryY + repY) / 2}, ${repX} ${repY - 36}"/>
        </circle>

        <!-- Replication Lag Badge -->
        <g transform="translate(${(primaryX + repX) / 2 - 35}, ${(primaryY + repY) / 2 - 12})">
          <rect width="70" height="22" rx="11" fill="${lagBadgeBg}" stroke="${lagBadgeBorder}" stroke-width="1.2"/>
          <text x="35" y="15" text-anchor="middle" fill="${strokeColor}" font-size="10" font-weight="700" font-family="JetBrains Mono">
            ${replica.lagMs}ms lag
          </text>
        </g>

        <!-- Replica Node Box -->
        <g transform="translate(${repX - 90}, ${repY - 32})" class="topo-node-g" style="cursor: pointer;">
          <rect width="180" height="74" rx="8" fill="${replicaBoxFill}" stroke="${strokeColor}" stroke-width="1.5"/>
          <text x="14" y="24" fill="${replicaTextPrimary}" font-size="12" font-weight="700" font-family="Inter">${replica.id}</text>
          <text x="14" y="42" fill="${replicaTextMuted}" font-size="10" font-family="Inter">${replica.ip}:${replica.port}</text>
          <text x="14" y="58" fill="${strokeColor}" font-size="10" font-weight="600" font-family="JetBrains Mono">${replica.role.toUpperCase()} STANDBY</text>
          <rect x="125" y="12" width="42" height="18" rx="4" fill="${strokeColor}22"/>
          <text x="146" y="25" text-anchor="middle" fill="${strokeColor}" font-size="9" font-weight="700">OK</text>
        </g>
      `;
    });

    // Draw Primary Node Box (Center Top)
    svgHtml += `
      <g transform="translate(${primaryX - 110}, ${primaryY - 36})" filter="url(#glowEffect)" style="cursor: pointer;">
        <rect width="220" height="84" rx="10" fill="url(#primaryGrad)"/>
        <text x="16" y="28" fill="#ffffff" font-size="14" font-weight="700" font-family="Inter">${primary.id}</text>
        <text x="16" y="48" fill="#e0f2fe" font-size="11" font-family="Inter">${primary.host}</text>
        <text x="16" y="68" fill="#ffffff" font-size="11" font-weight="800" font-family="JetBrains Mono">PRIMARY (WRITER)</text>
        <rect x="155" y="14" width="50" height="20" rx="4" fill="#ffffff33"/>
        <text x="180" y="28" text-anchor="middle" fill="#ffffff" font-size="10" font-weight="800">LEADER</text>
      </g>
      </svg>
    `;

    elements.topologySvgWrapper.innerHTML = svgHtml;
  }

  // --- Render Nodes Table ---
  function renderNodesTable() {
    const cluster = getActiveCluster();
    if (!cluster) return;

    elements.nodesTableBody.innerHTML = '';
    cluster.nodes.forEach(node => {
      const tr = document.createElement('tr');
      const isPrimary = node.role === 'primary';
      tr.innerHTML = `
        <td>
          <div style="font-weight: 600;">${node.id}</div>
          <div style="font-size: 11px; color: var(--text-muted);">${node.host}</div>
        </td>
        <td>
          <span class="role-pill ${node.role}">${node.role.toUpperCase()}</span>
        </td>
        <td>
          <span class="status-indicator text-emerald">
            <span class="pulse-indicator"></span> ${node.state.toUpperCase()}
          </span>
        </td>
        <td class="font-mono">${node.ip}:${node.port}</td>
        <td>
          <span style="font-weight: 600; color: ${node.lagMs === 0 ? 'var(--color-success)' : 'var(--brand-accent)'};">
            ${node.lagMs} ms
          </span>
          <span style="font-size: 11px; color: var(--text-muted);">(${node.lagBytes} B)</span>
        </td>
        <td class="font-mono" style="font-size: 11px;">${node.lsn}</td>
        <td>${node.cpu} / ${node.mem}</td>
        <td>
          ${isPrimary 
            ? '<button class="btn btn-secondary btn-sm" disabled title="Leader node">Leader</button>' 
            : `<button class="btn btn-secondary btn-sm" data-promote-node="${node.id}">Promote</button>`}
          <button class="btn btn-secondary btn-sm" data-fence-node="${node.id}">Fence</button>
        </td>
      `;

      const promoteBtn = tr.querySelector(`[data-promote-node="${node.id}"]`);
      if (promoteBtn) {
        promoteBtn.addEventListener('click', () => {
          openFailoverModalWithNode(node.id);
        });
      }

      const fenceBtn = tr.querySelector(`[data-fence-node="${node.id}"]`);
      if (fenceBtn) {
        fenceBtn.addEventListener('click', () => {
          openFenceModalWithNode(node.id);
        });
      }

      elements.nodesTableBody.appendChild(tr);
    });
  }

  // --- Render Candidates Table (HA & Recovery) ---
  function renderCandidates() {
    const cluster = getActiveCluster();
    if (!cluster) return;

    elements.candidatesTableBody.innerHTML = '';
    const standbys = cluster.nodes.filter(n => n.role !== 'primary');

    standbys.forEach((node, idx) => {
      const rank = idx + 1;
      const score = 100 - (node.lagMs * 1.5);
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td><strong style="color: var(--brand-accent);">#${rank}</strong></td>
        <td>
          <div style="font-weight: 600;">${node.id}</div>
          <div style="font-size: 11px; color: var(--text-muted);">${node.ip}</div>
        </td>
        <td class="font-mono">${node.lagMs} ms (${node.lagBytes} B)</td>
        <td class="font-mono" style="font-size: 11px;">${node.lsn}</td>
        <td><span class="status-indicator text-emerald"><span class="pulse-indicator"></span> HEALTHY</span></td>
        <td><strong class="font-mono">${score.toFixed(1)} / 100</strong></td>
        <td><span class="role-pill sync">QUALIFIED</span></td>
        <td>
          <button class="btn btn-primary btn-sm" data-select-candidate="${node.id}">Request Promotion</button>
        </td>
      `;

      tr.querySelector(`[data-select-candidate="${node.id}"]`).addEventListener('click', () => {
        openFailoverModalWithNode(node.id);
      });

      elements.candidatesTableBody.appendChild(tr);
    });
  }

  // --- Render Fencing Audit Table ---
  function renderFencingTable() {
    elements.fencingTableBody.innerHTML = '';
    state.fencing.forEach(item => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td class="font-mono">${item.id}</td>
        <td style="font-weight: 600;">${item.target}</td>
        <td><span class="role-pill fenced">${item.mechanism}</span></td>
        <td style="color: var(--text-secondary);">${item.reason}</td>
        <td>${item.actor}</td>
        <td><span class="status-indicator text-emerald">VERIFIED</span></td>
        <td style="color: var(--text-muted);">${item.timestamp}</td>
      `;
      elements.fencingTableBody.appendChild(tr);
    });
  }

  // --- Render Backups & Drills ---
  function renderBackupsAndDrills() {
    elements.backupsTableBody.innerHTML = '';
    state.backups.forEach(b => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td class="font-mono">${b.id}</td>
        <td style="font-weight: 600;">${b.cluster}</td>
        <td>${b.type}</td>
        <td>${b.engine}</td>
        <td class="font-mono">${b.size}</td>
        <td class="font-mono" style="font-size: 10px; max-width: 140px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${b.checksum}">
          ${b.checksum}
        </td>
        <td><span class="status-indicator text-emerald">SHA256 MATCH</span></td>
        <td style="color: var(--text-muted);">${b.completedAt}</td>
        <td>
          <button class="btn btn-secondary btn-sm" data-restore-backup="${b.id}">Run Drill</button>
        </td>
      `;
      tr.querySelector(`[data-restore-backup="${b.id}"]`).addEventListener('click', () => {
        showToast(`Started automated sandbox restore drill for ${b.id}...`, 'info');
      });
      elements.backupsTableBody.appendChild(tr);
    });

    elements.drillsTableBody.innerHTML = '';
    state.drills.forEach(d => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td class="font-mono">${d.id}</td>
        <td class="font-mono">${d.sourceBackup}</td>
        <td class="font-mono">${d.targetSandbox}</td>
        <td>${d.duration}</td>
        <td class="text-emerald">${d.corruption}</td>
        <td><span class="status-indicator text-emerald">${d.result}</span></td>
        <td style="color: var(--text-muted);">${d.completedAt}</td>
      `;
      elements.drillsTableBody.appendChild(tr);
    });
  }

  // --- Render Discovery Table ---
  function renderDiscovery() {
    elements.discoveryTableBody.innerHTML = '';
    state.discovery.forEach(d => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td class="font-mono">${d.id}</td>
        <td class="font-mono" style="font-weight: 600;">${d.host}</td>
        <td class="font-mono">${d.port}</td>
        <td><strong>${d.engine}</strong></td>
        <td>${d.version}</td>
        <td><span class="role-pill async">${d.role}</span></td>
        <td style="color: var(--text-muted);">${d.discoveredAt}</td>
        <td>
          <button class="btn btn-primary btn-sm" data-import-workload="${d.id}">Import to Nodlance</button>
        </td>
      `;
      tr.querySelector(`[data-import-workload="${d.id}"]`).addEventListener('click', () => {
        showToast(`Imported workload ${d.id} (${d.engine} ${d.version}) into Nodlance ClusterControl!`, 'success');
      });
      elements.discoveryTableBody.appendChild(tr);
    });
  }

  // --- Render Host Inventory ---
  function renderHosts() {
    elements.hostsTableBody.innerHTML = '';
    state.hosts.forEach(h => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td style="font-weight: 600;">${h.name}</td>
        <td class="font-mono">${h.ip}</td>
        <td>${h.os}</td>
        <td><span class="status-indicator text-emerald"><span class="pulse-indicator"></span> ${h.agentStatus}</span></td>
        <td class="font-mono">&lt; ${h.heartbeatSec}s ago</td>
        <td>
          <div style="display: flex; align-items: center; gap: 6px;">
            <div style="flex: 1; height: 6px; background: #1c2b44; border-radius: 3px; overflow: hidden;">
              <div style="width: ${h.cpu}%; height: 100%; background: var(--brand-accent);"></div>
            </div>
            <span class="font-mono" style="font-size: 11px;">${h.cpu}%</span>
          </div>
        </td>
        <td>
          <div style="display: flex; align-items: center; gap: 6px;">
            <div style="flex: 1; height: 6px; background: #1c2b44; border-radius: 3px; overflow: hidden;">
              <div style="width: ${h.ram}%; height: 100%; background: #10b981;"></div>
            </div>
            <span class="font-mono" style="font-size: 11px;">${h.ram}%</span>
          </div>
        </td>
        <td>
          <button class="btn btn-secondary btn-sm" data-ssh-host="${h.name}">Diagnose</button>
        </td>
      `;
      elements.hostsTableBody.appendChild(tr);
    });
  }

  // --- Render Alerts Table ---
  function renderAlerts() {
    elements.alertsTableBody.innerHTML = '';
    state.alerts.forEach((alt, idx) => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td><span class="role-pill" style="background: rgba(245, 158, 11, 0.2); color: #f59e0b; border: 1px solid #f59e0b;">${alt.severity}</span></td>
        <td style="font-weight: 600;">${alt.target}</td>
        <td>${alt.rule}</td>
        <td style="color: var(--text-secondary);">${alt.message}</td>
        <td style="color: var(--text-muted);">${alt.triggeredAt}</td>
        <td><span class="status-indicator text-amber">${alt.status}</span></td>
        <td>
          <button class="btn btn-secondary btn-sm" data-ack-alert="${idx}">Acknowledge</button>
        </td>
      `;
      tr.querySelector(`[data-ack-alert="${idx}"]`).addEventListener('click', () => {
        state.alerts.splice(idx, 1);
        renderAlerts();
        renderClusterCards();
        showToast('Alert acknowledged and silenced for maintenance.', 'success');
      });
      elements.alertsTableBody.appendChild(tr);
    });
  }

  // --- Render Audit Table ---
  function renderAudit() {
    elements.auditTableBody.innerHTML = '';
    state.audit.forEach(a => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td class="font-mono">${a.id}</td>
        <td class="font-mono" style="color: var(--text-muted);">${a.time}</td>
        <td style="font-weight: 600;">${a.actor}</td>
        <td><span class="role-pill primary">${a.action}</span></td>
        <td>${a.target}</td>
        <td><span class="status-indicator text-emerald">${a.status}</span></td>
        <td class="font-mono">${a.ip}</td>
      `;
      elements.auditTableBody.appendChild(tr);
    });
  }

  // --- Render Provisioning Plans ---
  function renderPlans() {
    elements.plansTableBody.innerHTML = '';
    state.plans.forEach(p => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td class="font-mono">${p.id}</td>
        <td style="font-weight: 600;">${p.cluster}</td>
        <td>${p.provider}</td>
        <td>${p.engine}</td>
        <td class="font-mono">${p.steps}</td>
        <td><span class="status-indicator text-emerald">${p.status}</span></td>
        <td style="color: var(--text-muted);">${p.created}</td>
        <td>
          <button class="btn btn-secondary btn-sm" data-view-plan="${p.id}">View Steps</button>
        </td>
      `;
      elements.plansTableBody.appendChild(tr);
    });
  }

  // --- Provisioning Wizard Engine (5 Steps) ---
  function updateWizardUI() {
    elements.wizardStepNum.textContent = state.wizard.step;
    elements.wizardProgressFill.style.width = `${(state.wizard.step / state.wizard.totalSteps) * 100}%`;

    elements.wizardSteps.forEach(step => {
      if (parseInt(step.dataset.step, 10) === state.wizard.step) {
        step.classList.add('active');
      } else {
        step.classList.remove('active');
      }
    });

    if (state.wizard.step > 1) {
      elements.wizBtnBack.style.display = 'inline-flex';
    } else {
      elements.wizBtnBack.style.display = 'none';
    }

    if (state.wizard.step === state.wizard.totalSteps) {
      elements.wizBtnNext.textContent = 'Execute Guarded Plan';
      elements.wizBtnNext.className = 'btn btn-primary';
    } else {
      elements.wizBtnNext.textContent = `Next: Step ${state.wizard.step + 1}`;
      elements.wizBtnNext.className = 'btn btn-primary';
    }
  }

  // Wizard Provider card selection
  document.querySelectorAll('input[name="wizProvider"]').forEach(radio => {
    radio.addEventListener('change', () => {
      document.querySelectorAll('.provider-option').forEach(o => o.classList.remove('selected'));
      radio.closest('.provider-option').classList.add('selected');
      state.wizard.provider = radio.value;
    });
  });

  // Wizard Engine card selection
  document.querySelectorAll('input[name="wizEngine"]').forEach(radio => {
    radio.addEventListener('change', () => {
      document.querySelectorAll('.engine-option').forEach(o => o.classList.remove('selected'));
      radio.closest('.engine-option').classList.add('selected');
      state.wizard.engine = radio.value;
    });
  });

  elements.wizBtnNext.addEventListener('click', () => {
    if (state.wizard.step < state.wizard.totalSteps) {
      state.wizard.step++;
      updateWizardUI();
    } else {
      // Step 5: Execute plan!
      elements.wizBtnNext.disabled = true;
      elements.wizBtnNext.textContent = 'Provisioning Cluster...';

      setTimeout(() => {
        const newClusterName = document.getElementById('wizClusterName').value || 'new-cluster';
        const newEngine = state.wizard.engine;
        const newEngineName = newEngine === 'postgres' ? 'PostgreSQL 16 HA' : newEngine === 'mysql' ? 'MySQL 8.4 LTS' : 'MariaDB 11.4 Galera';

        const newCluster = {
          id: `c${Date.now().toString(16)}`,
          name: newClusterName,
          engine: newEngine,
          engineName: newEngineName,
          provider: state.wizard.provider === 'linux' ? 'Linux Baremetal' : state.wizard.provider === 'docker' ? 'Docker Engine' : 'Kubernetes',
          status: 'healthy',
          qps: 180,
          avgLagMs: 2,
          storageGb: 40,
          nodesCount: 3,
          quorum: '3/3 Votes (Healthy)',
          epoch: 1,
          term: 1,
          nodes: [
            {
              id: `${newClusterName}-node-1`,
              host: `node-1.${newClusterName}.internal`,
              ip: '192.168.10.31',
              port: parseInt(document.getElementById('wizPort').value, 10) || 5432,
              role: 'primary',
              state: 'healthy',
              lagMs: 0,
              lagBytes: 0,
              lsn: '0/01000000',
              cpu: '5%',
              mem: '20%',
              uptime: '1m',
              cacheHitRatio: '100%'
            },
            {
              id: `${newClusterName}-node-2`,
              host: `node-2.${newClusterName}.internal`,
              ip: '192.168.10.32',
              port: parseInt(document.getElementById('wizPort').value, 10) || 5432,
              role: 'sync',
              state: 'healthy',
              lagMs: 2,
              lagBytes: 64,
              lsn: '0/01000000',
              cpu: '3%',
              mem: '18%',
              uptime: '1m',
              cacheHitRatio: '100%'
            },
            {
              id: `${newClusterName}-node-3`,
              host: `node-3.${newClusterName}.internal`,
              ip: '192.168.10.33',
              port: parseInt(document.getElementById('wizPort').value, 10) || 5432,
              role: 'async',
              state: 'healthy',
              lagMs: 3,
              lagBytes: 128,
              lsn: '0/01000000',
              cpu: '2%',
              mem: '18%',
              uptime: '1m',
              cacheHitRatio: '100%'
            }
          ]
        };

        state.clusters.push(newCluster);
        state.activeClusterId = newCluster.id;
        state.plans.unshift({
          id: `plan_${Date.now().toString(16)}`,
          cluster: newClusterName,
          provider: newCluster.provider,
          engine: newEngineName,
          steps: '5 / 5',
          status: 'COMPLETED (Guarded)',
          created: 'Just now'
        });

        populateClusterSelect();
        renderClusterCards();
        renderPlans();
        closeModal(elements.deployWizardModal);

        elements.wizBtnNext.disabled = false;
        elements.wizBtnNext.textContent = 'Next: Engine Selection';
        state.wizard.step = 1;
        updateWizardUI();

        showToast(`Cluster "${newClusterName}" successfully provisioned with guarded reverse rollback!`, 'success');
        switchView('topology');
      }, 1200);
    }
  });

  elements.wizBtnBack.addEventListener('click', () => {
    if (state.wizard.step > 1) {
      state.wizard.step--;
      updateWizardUI();
    }
  });

  // Modal Open Buttons
  elements.btnStartWizard.addEventListener('click', () => {
    state.wizard.step = 1;
    updateWizardUI();
    openModal(elements.deployWizardModal);
  });

  elements.btnOpenDeployWizard.addEventListener('click', () => {
    state.wizard.step = 1;
    updateWizardUI();
    openModal(elements.deployWizardModal);
  });

  elements.btnTestFailover.addEventListener('click', () => {
    openFailoverModal();
  });

  elements.btnRequestPromotion.addEventListener('click', () => {
    openFailoverModal();
  });

  elements.btnManualFenceModal.addEventListener('click', () => {
    openFenceModal();
  });

  elements.btnTriggerNodeBackup.addEventListener('click', () => {
    openBackupModal();
  });

  elements.btnCreateBackupModal.addEventListener('click', () => {
    openBackupModal();
  });

  elements.btnOpenDiscovery.addEventListener('click', () => {
    openModal(elements.scanModal);
  });

  elements.btnTriggerScanModal.addEventListener('click', () => {
    openModal(elements.scanModal);
  });

  elements.btnEnrollHostModal.addEventListener('click', () => {
    openModal(elements.enrollModal);
  });

  elements.btnLaunchFaultInjection.addEventListener('click', () => {
    showToast('Executing Fault Injection Drill: Simulating Primary node network isolation...', 'warning');
    setTimeout(() => {
      showToast('Fault Injection Test complete: Quorum held, Candidate #1 auto-promoted with 0 data loss.', 'success');
    }, 2000);
  });

  elements.btnRefreshFleet.addEventListener('click', () => {
    showToast('Refreshing cluster telemetry from Nodlance Controller...', 'info');
    renderClusterCards();
  });

  // Copy enroll command
  elements.btnCopyEnrollCmd.addEventListener('click', () => {
    const text = document.getElementById('enrollCmdText').textContent;
    navigator.clipboard.writeText(text).then(() => {
      showToast('Agent installation command copied to clipboard!', 'success');
    });
  });

  // Mode Switcher Button
  elements.modeToggleBtn.addEventListener('click', () => {
    state.isDemoMode = !state.isDemoMode;
    if (state.isDemoMode) {
      elements.modeText.textContent = 'Demo Fleet';
      elements.modeToggleBtn.style.borderColor = 'rgba(2, 132, 199, 0.4)';
      showToast('Switched to pre-seeded enterprise telemetry mode.', 'info');
    } else {
      elements.modeText.textContent = 'Live API';
      elements.modeToggleBtn.style.borderColor = 'rgba(5, 150, 105, 0.5)';
      showToast('Connected to live control plane backend on :8080.', 'success');
      fetchLiveClusters();
    }
  });

  // --- Theme Toggle Logic (Light Mode by Default, Dark as Option) ---
  const themeToggleBtn = document.getElementById('themeToggleBtn');
  const sunIcon = document.getElementById('sunIcon');
  const moonIcon = document.getElementById('moonIcon');
  const themeLabel = document.getElementById('themeLabel');

  function applyTheme(theme) {
    if (theme === 'dark') {
      document.body.classList.remove('light-theme');
      document.body.classList.add('dark-theme');
      if (sunIcon) sunIcon.style.display = 'none';
      if (moonIcon) moonIcon.style.display = 'inline-block';
      if (themeLabel) themeLabel.textContent = 'Dark';
      localStorage.setItem('nodlance_theme', 'dark');
    } else {
      document.body.classList.remove('dark-theme');
      document.body.classList.add('light-theme');
      if (sunIcon) sunIcon.style.display = 'inline-block';
      if (moonIcon) moonIcon.style.display = 'none';
      if (themeLabel) themeLabel.textContent = 'Light';
      localStorage.setItem('nodlance_theme', 'light');
    }
    renderTopology();
  }

  // Load saved theme or default to light
  const savedTheme = localStorage.getItem('nodlance_theme') || 'light';
  applyTheme(savedTheme);

  if (themeToggleBtn) {
    themeToggleBtn.addEventListener('click', () => {
      const isCurrentlyDark = document.body.classList.contains('dark-theme');
      applyTheme(isCurrentlyDark ? 'light' : 'dark');
      showToast(`Switched to ${isCurrentlyDark ? 'Light' : 'Dark'} mode.`, 'info');
    });
  }

  // Failover Modal Logic
  function openFailoverModalWithNode(nodeId) {
    populateCandidateDropdown();
    if (nodeId) {
      document.getElementById('promoteCandidateSelect').value = nodeId;
    }
    openModal(elements.failoverModal);
  }

  function openFailoverModal() {
    populateCandidateDropdown();
    openModal(elements.failoverModal);
  }

  function populateCandidateDropdown() {
    const cluster = getActiveCluster();
    const select = document.getElementById('promoteCandidateSelect');
    select.innerHTML = '';
    cluster.nodes.filter(n => n.role !== 'primary').forEach(n => {
      const opt = document.createElement('option');
      opt.value = n.id;
      opt.textContent = `${n.id} (${n.ip} - Lag: ${n.lagMs}ms - Score: ${(100 - n.lagMs * 1.5).toFixed(1)})`;
      select.appendChild(opt);
    });
  }

  elements.btnRegenToken.addEventListener('click', () => {
    const randHex = Math.random().toString(16).substring(2, 10) + Math.random().toString(16).substring(2, 10);
    document.getElementById('promoteTokenInput').value = `tok_prom_${randHex}`;
    showToast('New cryptographic operator approval token generated.', 'info');
  });

  elements.btnExecutePromotion.addEventListener('click', () => {
    const targetCandidateId = document.getElementById('promoteCandidateSelect').value;
    const cluster = getActiveCluster();
    const oldPrimary = cluster.nodes.find(n => n.role === 'primary');
    const newPrimary = cluster.nodes.find(n => n.id === targetCandidateId);

    if (oldPrimary && newPrimary) {
      oldPrimary.role = 'fenced';
      oldPrimary.state = 'isolated';
      newPrimary.role = 'primary';
      newPrimary.lagMs = 0;
      newPrimary.lagBytes = 0;
      cluster.epoch++;
      cluster.term++;

      state.fencing.unshift({
        id: `fnc_${Date.now().toString(16)}`,
        target: `${oldPrimary.id} (${oldPrimary.ip})`,
        mechanism: 'STONITH (Power Off via IPMI)',
        reason: `Operator promotion of ${newPrimary.id} with token`,
        actor: 'admin (Superadmin)',
        status: 'VERIFIED ISOLATED',
        timestamp: 'Just now'
      });

      state.audit.unshift({
        id: `aud_${Date.now().toString(16)}`,
        time: new Date().toISOString().substring(11, 19) + ' UTC',
        actor: 'admin (Superadmin)',
        action: 'PROMOTION_OPERATOR_APPROVED',
        target: `${cluster.name} -> ${newPrimary.id} (Epoch #${cluster.epoch})`,
        status: 'SUCCESS',
        ip: '10.0.4.12'
      });

      renderTopology();
      renderNodesTable();
      renderCandidates();
      renderFencingTable();
      renderAudit();
      closeModal(elements.failoverModal);

      showToast(`Candidate ${newPrimary.id} successfully promoted to Primary! Old primary isolated via STONITH.`, 'success');
    }
  });

  // Fencing Modal Logic
  function openFenceModalWithNode(nodeId) {
    populateFenceNodeDropdown();
    if (nodeId) {
      document.getElementById('fenceNodeSelect').value = nodeId;
    }
    openModal(elements.fenceModal);
  }

  function openFenceModal() {
    populateFenceNodeDropdown();
    openModal(elements.fenceModal);
  }

  function populateFenceNodeDropdown() {
    const cluster = getActiveCluster();
    const select = document.getElementById('fenceNodeSelect');
    select.innerHTML = '';
    cluster.nodes.forEach(n => {
      const opt = document.createElement('option');
      opt.value = n.id;
      opt.textContent = `${n.id} (${n.ip}:${n.port}) - ${n.role.toUpperCase()}`;
      select.appendChild(opt);
    });
  }

  elements.btnConfirmFence.addEventListener('click', () => {
    const nodeId = document.getElementById('fenceNodeSelect').value;
    const mechanism = document.getElementById('fenceMechanismSelect').value.toUpperCase();
    const reason = document.getElementById('fenceReasonInput').value;
    const cluster = getActiveCluster();
    const node = cluster.nodes.find(n => n.id === nodeId);

    if (node) {
      node.role = 'fenced';
      node.state = 'isolated';

      state.fencing.unshift({
        id: `fnc_${Date.now().toString(16)}`,
        target: `${node.id} (${node.ip})`,
        mechanism: mechanism,
        reason: reason,
        actor: 'admin (Superadmin)',
        status: 'VERIFIED ISOLATED',
        timestamp: 'Just now'
      });

      renderTopology();
      renderNodesTable();
      renderFencingTable();
      closeModal(elements.fenceModal);
      showToast(`Node ${node.id} isolated via ${mechanism}. Split-brain hazard eliminated!`, 'success');
    }
  });

  // Backup Modal Logic
  function openBackupModal() {
    const select = document.getElementById('backupClusterSelect');
    select.innerHTML = '';
    state.clusters.forEach(c => {
      const opt = document.createElement('option');
      opt.value = c.id;
      opt.textContent = c.name;
      if (c.id === state.activeClusterId) opt.selected = true;
      select.appendChild(opt);
    });
    openModal(elements.backupModal);
  }

  elements.btnSubmitBackup.addEventListener('click', () => {
    const clusterId = document.getElementById('backupClusterSelect').value;
    const cluster = state.clusters.find(c => c.id === clusterId) || state.clusters[0];
    const bType = document.getElementById('backupTypeSelect').value;
    const bId = `bk_${Date.now().toString(16)}`;

    closeModal(elements.backupModal);
    showToast(`Backup job ${bId} started for cluster ${cluster.name}...`, 'info');

    setTimeout(() => {
      state.backups.unshift({
        id: bId,
        cluster: cluster.name,
        type: bType === 'full' ? 'Full Physical Snapshot' : 'Incremental Stream',
        engine: cluster.engineName,
        size: '14.2 GB',
        checksum: 'b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9',
        status: 'VERIFIED',
        completedAt: 'Just now'
      });

      state.drills.unshift({
        id: `drill_${Date.now().toString(16)}`,
        sourceBackup: bId,
        targetSandbox: `sandbox-${cluster.engine}-drill.tmp`,
        duration: '29s',
        corruption: '0 blocks',
        result: 'PASSED (100% Verified)',
        completedAt: 'Just now'
      });

      renderBackupsAndDrills();
      showToast(`Backup ${bId} verified and validated with automated drill!`, 'success');
    }, 1200);
  });

  // Discovery Scan Logic
  elements.btnRunScan.addEventListener('click', () => {
    const cidr = document.getElementById('scanCidrInput').value;
    closeModal(elements.scanModal);
    showToast(`Scanning subnet ${cidr} on ports 5432, 3306, 4567...`, 'info');

    setTimeout(() => {
      state.discovery.unshift({
        id: `disc_workload_${Date.now().toString(16)}`,
        host: '192.168.10.77',
        port: 5432,
        engine: 'PostgreSQL',
        version: '16.2',
        role: 'Primary / Standalone',
        discoveredAt: 'Just now'
      });
      renderDiscovery();
      showToast(`Scan complete! 1 unmanaged database discovered on 192.168.10.77:5432.`, 'success');
      switchView('discovery');
    }, 1500);
  });

  // Live API Fetcher
  async function fetchLiveClusters() {
    try {
      const res = await fetch('/api/v1/clusters');
      if (res.ok) {
        const data = await res.json();
        if (Array.isArray(data) && data.length > 0) {
          state.clusters = data;
          state.activeClusterId = data[0].id;
          populateClusterSelect();
          renderClusterCards();
          renderTopology();
          renderNodesTable();
        }
      }
    } catch (_) {
      // Gracefully stays in pre-seeded mode
    }
  }

  // Periodic heartbeat animation
  setInterval(() => {
    state.hosts.forEach(h => {
      h.heartbeatSec = Math.floor(Math.random() * 3) + 1;
    });
    if (state.activeView === 'hosts') {
      renderHosts();
    }
  }, 3000);

  // --- Initial Mount ---
  function init() {
    populateClusterSelect();
    renderClusterCards();
    renderBackupsAndDrills();
    renderDiscovery();
    renderHosts();
    renderAlerts();
    renderAudit();
    renderPlans();

    // Check if backend API has clusters
    fetchLiveClusters();
  }

  // Run on DOM ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
