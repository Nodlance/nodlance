/**
 * Nodlance ClusterControl - Pre-seeded Enterprise Fleet Data
 */

export const INITIAL_CLUSTERS = [
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

export const INITIAL_HOSTS = [
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

export const INITIAL_BACKUPS = [
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

export const INITIAL_DRILLS = [
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

export const INITIAL_DISCOVERY = [
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

export const INITIAL_ALERTS = [
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

export const INITIAL_AUDIT = [
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

export const INITIAL_FENCING = [
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

export const INITIAL_PLANS = [
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
