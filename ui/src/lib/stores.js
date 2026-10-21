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
  INITIAL_PLANS
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
