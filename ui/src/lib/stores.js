import { writable, derived } from 'svelte/store';
import {
  INITIAL_CLUSTERS,
  INITIAL_HOSTS,
  INITIAL_BACKUPS,
  INITIAL_DRILLS,
  INITIAL_DISCOVERY,
  INITIAL_ALERTS,
  INITIAL_AUDIT,
  INITIAL_FENCING,
  INITIAL_PLANS,
  INITIAL_LOAD_BALANCERS,
  INITIAL_BACKUP_SCHEDULES,
  INITIAL_PITR_STREAMS,
  INITIAL_ADVISOR_RULES,
  INITIAL_ADVISOR_FINDINGS,
  INITIAL_SLOW_QUERIES,
  INITIAL_CONFIG_TEMPLATES,
  INITIAL_PKI_CERTIFICATES
} from './data.js';

// --- Theme Store (Light default, Dark optional) ---
const savedTheme = typeof localStorage !== 'undefined' ? (localStorage.getItem('nodlance_theme') || 'light') : 'light';
export const theme = writable(savedTheme);

theme.subscribe((val) => {
  if (typeof document !== 'undefined') {
    if (val === 'dark') {
      document.body.classList.remove('light-theme');
      document.body.classList.add('dark-theme');
    } else {
      document.body.classList.remove('dark-theme');
      document.body.classList.add('light-theme');
    }
    localStorage.setItem('nodlance_theme', val);
  }
});

export function toggleTheme() {
  theme.update(current => (current === 'dark' ? 'light' : 'dark'));
}

// --- Active View Navigation ---
export const activeView = writable('fleet');
export function switchView(v) {
  activeView.set(v);
}

// --- Core Data Stores ---
export const clusters = writable(JSON.parse(JSON.stringify(INITIAL_CLUSTERS)));
export const activeClusterId = writable('c1a7001-pg16-prod');
export const engineFilter = writable('all');
export const isDemoMode = writable(true);

export const activeCluster = derived(
  [clusters, activeClusterId],
  ([$clusters, $id]) => $clusters.find(c => c.id === $id) || $clusters[0]
);

export const hosts = writable(JSON.parse(JSON.stringify(INITIAL_HOSTS)));
export const backups = writable(JSON.parse(JSON.stringify(INITIAL_BACKUPS)));
export const drills = writable(JSON.parse(JSON.stringify(INITIAL_DRILLS)));
export const discovery = writable(JSON.parse(JSON.stringify(INITIAL_DISCOVERY)));
export const alerts = writable(JSON.parse(JSON.stringify(INITIAL_ALERTS)));
export const audit = writable(JSON.parse(JSON.stringify(INITIAL_AUDIT)));
export const fencing = writable(JSON.parse(JSON.stringify(INITIAL_FENCING)));
export const plans = writable(JSON.parse(JSON.stringify(INITIAL_PLANS)));
export const loadBalancers = writable(JSON.parse(JSON.stringify(INITIAL_LOAD_BALANCERS)));
export const backupSchedules = writable(JSON.parse(JSON.stringify(INITIAL_BACKUP_SCHEDULES)));
export const pitrStreams = writable(JSON.parse(JSON.stringify(INITIAL_PITR_STREAMS)));
export const advisorRules = writable(JSON.parse(JSON.stringify(INITIAL_ADVISOR_RULES)));
export const advisorFindings = writable(JSON.parse(JSON.stringify(INITIAL_ADVISOR_FINDINGS)));
export const slowQueries = writable(JSON.parse(JSON.stringify(INITIAL_SLOW_QUERIES)));

// --- Toasts ---
export const toasts = writable([]);
export function showToast(message, type = 'info') {
  const id = Date.now().toString(16) + Math.random().toString(16).slice(2, 6);
  toasts.update(t => [...t, { id, message, type }]);
  setTimeout(() => {
    toasts.update(t => t.filter(item => item.id !== id));
  }, 3500);
}

// --- Modals State ---
/** @type {import('svelte/store').Writable<string|null>} */
export const activeModal = writable(null); // 'deployWizard' | 'failover' | 'fence' | 'backup' | 'scan' | 'enroll'
/** @type {import('svelte/store').Writable<any>} */
export const modalContextNode = writable(null);

export function openModal(name, node = null) {
  modalContextNode.set(node);
  activeModal.set(name);
}

export function closeModal() {
  activeModal.set(null);
  modalContextNode.set(null);
}

// --- Actions ---
export function promoteCandidate(candidateId) {
  clusters.update($clusters => {
    return $clusters.map(cluster => {
      const candidate = cluster.nodes.find(n => n.id === candidateId);
      if (!candidate) return cluster;

      const oldPrimary = cluster.nodes.find(n => n.role === 'primary');
      if (oldPrimary) {
        oldPrimary.role = 'fenced';
        oldPrimary.state = 'isolated';
      }

      candidate.role = 'primary';
      candidate.lagMs = 0;
      candidate.lagBytes = 0;
      cluster.epoch = (cluster.epoch || 1) + 1;
      cluster.term = (cluster.term || 1) + 1;

      // Add fencing audit
      if (oldPrimary) {
        fencing.update(f => [
          {
            id: `fnc_${Date.now().toString(16)}`,
            target: `${oldPrimary.id} (${oldPrimary.ip})`,
            mechanism: 'STONITH (Power Off via IPMI)',
            reason: `Operator promotion of ${candidate.id}`,
            actor: 'admin (Superadmin)',
            status: 'VERIFIED ISOLATED',
            timestamp: 'Just now'
          },
          ...f
        ]);
      }

      // Add audit log
      audit.update(a => [
        {
          id: `aud_${Date.now().toString(16)}`,
          time: new Date().toISOString().substring(11, 19) + ' UTC',
          actor: 'admin (Superadmin)',
          action: 'PROMOTION_OPERATOR_APPROVED',
          target: `${cluster.name} -> ${candidate.id} (Epoch #${cluster.epoch})`,
          status: 'SUCCESS',
          ip: '10.0.4.12'
        },
        ...a
      ]);

      return cluster;
    });
  });
  showToast(`Candidate ${candidateId} promoted to Primary successfully!`, 'success');
}

export function fenceNode(nodeId, mechanism, reason) {
  clusters.update($clusters => {
    return $clusters.map(cluster => {
      const node = cluster.nodes.find(n => n.id === nodeId);
      if (node) {
        node.role = 'fenced';
        node.state = 'isolated';

        fencing.update(f => [
          {
            id: `fnc_${Date.now().toString(16)}`,
            target: `${node.id} (${node.ip})`,
            mechanism: mechanism.toUpperCase(),
            reason: reason || 'Suspected split-brain isolation',
            actor: 'admin (Superadmin)',
            status: 'VERIFIED ISOLATED',
            timestamp: 'Just now'
          },
          ...f
        ]);
      }
      return cluster;
    });
  });
  showToast(`Node ${nodeId} isolated via ${mechanism.toUpperCase()}.`, 'success');
}

export function triggerClusterBackup(clusterId, bType) {
  const bId = `bk_${Date.now().toString(16)}`;
  showToast(`Backup job ${bId} initiated...`, 'info');

  setTimeout(() => {
    clusters.subscribe($clusters => {
      const cluster = $clusters.find(c => c.id === clusterId) || $clusters[0];
      backups.update(b => [
        {
          id: bId,
          cluster: cluster.name,
          type: bType === 'full' ? 'Full Physical Snapshot' : 'Incremental Stream',
          engine: cluster.engineName,
          size: '14.2 GB',
          checksum: 'b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9',
          status: 'VERIFIED',
          completedAt: 'Just now'
        },
        ...b
      ]);

      drills.update(d => [
        {
          id: `drill_${Date.now().toString(16)}`,
          sourceBackup: bId,
          targetSandbox: `sandbox-${cluster.engine}-drill.tmp`,
          duration: '29s',
          corruption: '0 blocks',
          result: 'PASSED (100% Verified)',
          completedAt: 'Just now'
        },
        ...d
      ]);
    })();
    showToast(`Backup ${bId} verified and validated with automated drill!`, 'success');
  }, 1200);
}

export function runDiscoveryScan(cidr) {
  showToast(`Scanning subnet ${cidr} on database ports...`, 'info');
  setTimeout(() => {
    discovery.update(d => [
      {
        id: `disc_workload_${Date.now().toString(16)}`,
        host: '192.168.10.77',
        port: 5432,
        engine: 'PostgreSQL',
        version: '16.2',
        role: 'Primary / Standalone',
        discoveredAt: 'Just now'
      },
      ...d
    ]);
    showToast('Discovery scan completed! 1 instance found.', 'success');
  }, 1400);
}

export function importDiscoveredWorkload(id) {
  showToast(`Workload ${id} imported into Nodlance ClusterControl!`, 'success');
}

export function acknowledgeAlert(index) {
  alerts.update(a => a.filter((_, i) => i !== index));
  showToast('Alert acknowledged and suppressed.', 'success');
}

export function deployNewCluster(wizData) {
  const newCluster = {
    id: `c${Date.now().toString(16)}`,
    name: wizData.clusterName || 'new-db-cluster',
    engine: wizData.engine,
    engineName: wizData.engine === 'postgres' ? 'PostgreSQL 16 HA' : wizData.engine === 'mysql' ? 'MySQL 8.4 LTS' : 'MariaDB 11.4 Galera',
    provider: wizData.provider === 'linux' ? 'Linux Baremetal' : wizData.provider === 'docker' ? 'Docker Engine' : 'Kubernetes',
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
        id: `${wizData.clusterName}-node-1`,
        host: `node-1.${wizData.clusterName}.internal`,
        ip: '192.168.10.31',
        port: wizData.port || 5432,
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
        id: `${wizData.clusterName}-node-2`,
        host: `node-2.${wizData.clusterName}.internal`,
        ip: '192.168.10.32',
        port: wizData.port || 5432,
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
        id: `${wizData.clusterName}-node-3`,
        host: `node-3.${wizData.clusterName}.internal`,
        ip: '192.168.10.33',
        port: wizData.port || 5432,
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

  clusters.update(c => [...c, newCluster]);
  activeClusterId.set(newCluster.id);
  plans.update(p => [
    {
      id: `plan_${Date.now().toString(16)}`,
      cluster: newCluster.name,
      provider: newCluster.provider,
      engine: newCluster.engineName,
      steps: '5 / 5',
      status: 'COMPLETED (Guarded)',
      created: 'Just now'
    },
    ...p
  ]);

  showToast(`Cluster ${newCluster.name} provisioned with guarded reverse rollback!`, 'success');
  activeView.set('topology');
}

// Load Balancer Actions
export async function reloadLoadBalancer(id) {
  try {
    const res = await fetch(`/api/v1/load-balancers/${id}/reload`, { method: 'POST' });
    if (res.ok) {
      const data = await res.json();
      showToast(data.message || 'Load balancer configuration reloaded and synchronized!', 'success');
    }
  } catch (_) {
    // Demo fallback
  }

  loadBalancers.update(list => list.map(lb => lb.id === id ? { ...lb, status: 'active', updated: 'Just now' } : lb));
  showToast('Proxy rules synchronized to runtime and disk!', 'success');

  audit.update(a => [
    {
      id: `aud_${Date.now().toString(16).slice(-4)}`,
      time: new Date().toISOString().slice(11, 19) + ' UTC',
      actor: 'admin (Superadmin)',
      action: 'LOAD_BALANCER_RELOAD',
      target: `LB ${id}`,
      status: 'SUCCESS',
      ip: '10.0.4.12'
    },
    ...a
  ]);
}

export function createLoadBalancer(lbData) {
  const newLb = {
    id: `lb_${Date.now().toString(16).slice(-6)}`,
    name: lbData.name || 'New Load Balancer',
    clusterId: lbData.clusterId || 'c1a7001-pg16-prod',
    clusterName: lbData.clusterName || 'prod-pg16-ha-primary',
    type: lbData.type || 'proxysql',
    host: lbData.host || 'node-01.internal',
    ip: lbData.ip || '192.168.10.50',
    listenPort: parseInt(lbData.listenPort, 10) || 6033,
    adminPort: lbData.adminPort ? parseInt(lbData.adminPort, 10) : null,
    status: 'active',
    mode: lbData.type === 'proxysql' ? 'Read/Write Split (HG 10/20)' : (lbData.type === 'pgbouncer' ? 'Transaction Pooling' : 'VRRP Master'),
    writers: 'Primary Node',
    readers: 'Replica Pool',
    qps: 0,
    activeConnections: 0,
    cacheHitRatio: '100%',
    updated: 'Just now'
  };

  loadBalancers.update(l => [newLb, ...l]);
  showToast(`Load balancer ${newLb.name} (${newLb.type}) deployed successfully!`, 'success');
  activeModal.set(null);
}

// Backup Schedule & PITR Actions
export function createBackupSchedule(data) {
  const newSchedule = {
    id: `sched_${Date.now().toString(16).slice(-6)}`,
    clusterId: data.clusterId || 'c1a7001-pg16-prod',
    clusterName: data.clusterName || 'prod-pg16-ha-primary',
    name: data.name || 'Automated Cluster Backup',
    cronExpression: data.cronExpression || '0 2 * * *',
    scheduleHuman: data.scheduleHuman || 'Daily at 02:00 AM UTC',
    backupMethod: data.backupMethod || 'pg_basebackup',
    storageTarget: data.storageTarget || 's3',
    storagePath: data.storagePath || 's3://nodlance-backups/cluster/',
    retentionDays: parseInt(data.retentionDays, 10) || 30,
    compression: data.compression || 'zstd',
    encrypt: data.encrypt !== false,
    isActive: true,
    lastRunStatus: 'PENDING',
    lastRunAt: 'Never',
    nextRunAt: 'Scheduled'
  };

  backupSchedules.update(s => [newSchedule, ...s]);
  showToast(`Backup schedule "${newSchedule.name}" configured and activated!`, 'success');
  activeModal.set(null);
}

export function deleteBackupSchedule(id) {
  backupSchedules.update(s => s.filter(item => item.id !== id));
  showToast('Backup schedule deleted.', 'info');
}

export function toggleBackupSchedule(id) {
  backupSchedules.update(s => s.map(item => {
    if (item.id === id) {
      const updated = !item.isActive;
      showToast(`Schedule "${item.name}" ${updated ? 'resumed' : 'paused'}.`, 'info');
      return { ...item, isActive: updated };
    }
    return item;
  }));
}

export function rewindPitrStream(clusterId, timestampStr) {
  showToast(`Initiating PITR recovery validation rewind for ${clusterId} to target: ${timestampStr}`, 'info');
}

// Advisor Actions
export function runAdvisorScan(clusterId) {
  showToast(`Running embedded Rhai DBA Heuristic Scan on cluster ${clusterId}...`, 'info');
  setTimeout(() => {
    showToast(`DBA Scan complete. 3 findings evaluated against production policies.`, 'success');
  }, 1200);
}

export function resolveAdvisorFinding(id) {
  advisorFindings.update(f => f.filter(item => item.id !== id));
  showToast(`Advisor finding marked as resolved.`, 'success');
}

export function toggleAdvisorRule(id) {
  advisorRules.update(r => r.map(item => {
    if (item.id === id) {
      const updated = !item.isEnabled;
      showToast(`Advisor rule "${item.name}" ${updated ? 'enabled' : 'disabled'}.`, 'info');
      return { ...item, isEnabled: updated };
    }
    return item;
  }));
}

// Live API Poll
export async function fetchLiveClusters() {
  try {
    const res = await fetch('/api/v1/clusters');
    if (res.ok) {
      const data = await res.json();
      if (Array.isArray(data) && data.length > 0) {
        clusters.set(data);
        activeClusterId.set(data[0].id);
      }
    }
  } catch (_) {
    // Falls back gracefully to pre-seeded telemetry
  }
}

// Config & PKI Stores
export const configTemplates = writable(JSON.parse(JSON.stringify(INITIAL_CONFIG_TEMPLATES)));
export const pkiCertificates = writable(JSON.parse(JSON.stringify(INITIAL_PKI_CERTIFICATES)));

export function revokeCertificate(id) {
  pkiCertificates.update(certs => certs.map(c => c.id === id ? { ...c, isRevoked: true, status: 'REVOKED' } : c));
  showToast('X.509 Certificate successfully revoked and flagged in CRL.', 'warning');
}

export function issueCertificate(certData) {
  const newCert = {
    id: `cert_${Date.now()}`,
    clusterId: certData.clusterId || null,
    clusterName: certData.clusterName || 'Global Cluster',
    commonName: certData.commonName,
    certType: certData.certType || 'server',
    fingerprintSha256: Array.from({length: 64}, () => Math.floor(Math.random()*16).toString(16)).join(''),
    validFrom: new Date().toISOString().split('T')[0],
    expiresAt: new Date(Date.now() + (certData.validityDays || 365)*86400000).toISOString().split('T')[0],
    daysLeft: certData.validityDays || 365,
    isRevoked: false,
    status: 'ACTIVE'
  };
  pkiCertificates.update(certs => [newCert, ...certs]);
  showToast(`Certificate issued for ${newCert.commonName}`, 'success');
}

export function updateConfigTemplate(templateId, newContent) {
  configTemplates.update(tmpls => tmpls.map(t => t.id === templateId ? { ...t, content: newContent, version: t.version + 1, updatedAt: 'Just now' } : t));
  showToast('Configuration template updated successfully.', 'success');
}


