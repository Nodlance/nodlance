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

export const INITIAL_LOAD_BALANCERS = [
  {
    id: 'lb_pxsql_01',
    name: 'ProxySQL Core Router',
    clusterId: 'c1a7002-my80-prod',
    clusterName: 'core-mysql-ecommerce',
    type: 'proxysql',
    host: 'proxy-01.prod.internal',
    ip: '192.168.20.15',
    listenPort: 6033,
    adminPort: 6032,
    status: 'active',
    mode: 'Read/Write Split (HG 10/20)',
    writers: '1 (mysql-master-01)',
    readers: '2 (mysql-replica-01, mysql-replica-02)',
    qps: 18450,
    activeConnections: 312,
    cacheHitRatio: '98.8%',
    updated: 'Just now'
  },
  {
    id: 'lb_pgb_01',
    name: 'PgBouncer Transaction Pooler',
    clusterId: 'c1a7001-pg16-prod',
    clusterName: 'prod-pg16-ha-primary',
    type: 'pgbouncer',
    host: 'db-pg-node-01.prod.internal',
    ip: '192.168.10.21',
    listenPort: 6432,
    adminPort: 6433,
    status: 'active',
    mode: 'Transaction Pooling',
    writers: '1 (db-pg-node-01)',
    readers: '2 (db-pg-node-02, db-pg-node-03)',
    qps: 12100,
    activeConnections: 640,
    cacheHitRatio: '99.2%',
    updated: '2m ago'
  },
  {
    id: 'lb_vrrp_01',
    name: 'Keepalived Virtual IP (VIP)',
    clusterId: 'c1a7001-pg16-prod',
    clusterName: 'prod-pg16-ha-primary',
    type: 'keepalived',
    host: 'db-pg-node-01.prod.internal',
    ip: '192.168.10.100 (VIP)',
    listenPort: 5432,
    adminPort: null,
    status: 'active',
    mode: 'VRRP Master (VRID 51)',
    writers: 'Primary Node 192.168.10.21',
    readers: 'VRRP Standby on Node 02',
    qps: 12100,
    activeConnections: 640,
    cacheHitRatio: '100%',
    updated: 'Just now'
  }
];

export const INITIAL_BACKUP_SCHEDULES = [
  {
    id: 'sched_pg_daily_s3',
    clusterId: 'c1a7001-pg16-prod',
    clusterName: 'prod-pg16-ha-primary',
    name: 'Daily Production Full to AWS S3',
    cronExpression: '0 2 * * *',
    scheduleHuman: 'Daily at 02:00 AM UTC',
    backupMethod: 'pg_basebackup',
    storageTarget: 's3',
    storagePath: 's3://nodlance-backups/pg16-prod/',
    retentionDays: 30,
    compression: 'zstd',
    encrypt: true,
    isActive: true,
    lastRunStatus: 'SUCCESS',
    lastRunAt: 'Today, 02:00 AM',
    nextRunAt: 'Tomorrow, 02:00 AM'
  },
  {
    id: 'sched_mysql_hourly_minio',
    clusterId: 'c2b8002-my84-prod',
    clusterName: 'core-mysql-ecommerce',
    name: 'Hourly Incremental to MinIO',
    cronExpression: '0 * * * *',
    scheduleHuman: 'Hourly at minute 0',
    backupMethod: 'xtrabackup',
    storageTarget: 'minio',
    storagePath: 'minio://infra-minio:9000/backups/mysql-ecom/',
    retentionDays: 14,
    compression: 'lz4',
    encrypt: true,
    isActive: true,
    lastRunStatus: 'SUCCESS',
    lastRunAt: '38 mins ago',
    nextRunAt: 'in 22 mins'
  },
  {
    id: 'sched_galera_weekly_gcs',
    clusterId: 'c3c9003-ma11-galera',
    clusterName: 'bi-reporting-galera',
    name: 'Weekly Full Archive to GCS',
    cronExpression: '0 3 * * 0',
    scheduleHuman: 'Every Sunday at 03:00 AM UTC',
    backupMethod: 'mariabackup',
    storageTarget: 'gcs',
    storagePath: 'gs://nodlance-cold-archives/galera-bi/',
    retentionDays: 90,
    compression: 'zstd',
    encrypt: true,
    isActive: true,
    lastRunStatus: 'SUCCESS',
    lastRunAt: '4 days ago',
    nextRunAt: 'in 3 days'
  }
];

export const INITIAL_PITR_STREAMS = [
  {
    clusterId: 'c1a7001-pg16-prod',
    clusterName: 'prod-pg16-ha-primary',
    storageTarget: 's3',
    storagePath: 's3://nodlance-backups/pg16-prod/wal/',
    status: 'ACTIVE_STREAMING',
    startTime: '2026-10-15T00:00:00Z',
    latestTime: '2026-10-22T08:29:45Z',
    startLsn: '0/1A0000',
    latestLsn: '0/4F9B200',
    rpoLagSeconds: 15,
    recoveryWindow: '7 days continuous rewind'
  },
  {
    clusterId: 'c2b8002-my84-prod',
    clusterName: 'core-mysql-ecommerce',
    storageTarget: 'minio',
    storagePath: 'minio://infra-minio:9000/backups/mysql-ecom/binlog/',
    status: 'ACTIVE_STREAMING',
    startTime: '2026-10-18T00:00:00Z',
    latestTime: '2026-10-22T08:29:10Z',
    startLsn: 'binlog.000120:4',
    latestLsn: 'binlog.000188:94102',
    rpoLagSeconds: 25,
    recoveryWindow: '4 days continuous rewind'
  }
];

export const INITIAL_ADVISOR_RULES = [
  {
    id: 'rule_innodb_buffer_pool',
    name: 'Buffer Pool Cache Hit Ratio Low',
    engine: 'mysql',
    category: 'performance',
    severity: 'warning',
    description: 'InnoDB buffer pool hit ratio is below 95%. Consider increasing innodb_buffer_pool_size to reduce disk read IOPS.',
    scriptContent: 'buffer_pool_hit_ratio < 0.95',
    isEnabled: true
  },
  {
    id: 'rule_connection_ceiling',
    name: 'High Connection Saturation (>80%)',
    engine: 'any',
    category: 'capacity',
    severity: 'critical',
    description: 'Active connections exceed 80% of max_connections limit. Risk of connection starvation. Deploy PgBouncer or ProxySQL connection pooling.',
    scriptContent: '(active_connections * 100) / max_connections > 80',
    isEnabled: true
  },
  {
    id: 'rule_autovacuum_bloat',
    name: 'Table Bloat & Autovacuum Lag',
    engine: 'postgres',
    category: 'capacity',
    severity: 'warning',
    description: 'Dead tuples ratio exceeds 15%. Autovacuum workers may be lagging. Check autovacuum_vacuum_scale_factor and autovacuum_max_workers.',
    scriptContent: 'dead_tuples_ratio > 0.15',
    isEnabled: true
  },
  {
    id: 'rule_missing_primary_keys',
    name: 'Tables Without Primary Keys',
    engine: 'any',
    category: 'availability',
    severity: 'critical',
    description: 'Discovered tables lacking primary keys. This breaks row-based replication consistency and Galera multi-primary replication certification.',
    scriptContent: 'missing_primary_keys_count > 0',
    isEnabled: true
  },
  {
    id: 'rule_non_tls_listener',
    name: 'Unencrypted Plaintext Database Traffic',
    engine: 'any',
    category: 'security',
    severity: 'critical',
    description: 'Cluster is operating without enforced TLS/SSL encryption. Database passwords and sensitive data are vulnerable to network snooping. Enable mTLS.',
    scriptContent: '!tls_enabled',
    isEnabled: true
  },
  {
    id: 'rule_galera_flow_control',
    name: 'Galera Cluster Flow Control Paused Stalling',
    engine: 'galera',
    category: 'availability',
    severity: 'critical',
    description: 'wsrep_flow_control_paused exceeds 5%. One or more Galera nodes cannot keep up with write rate, causing replication pauses across the entire cluster.',
    scriptContent: 'galera_flow_control_paused_pct > 5.0',
    isEnabled: true
  }
];

export const INITIAL_ADVISOR_FINDINGS = [
  {
    id: 'fnd_01_bloat',
    ruleId: 'rule_autovacuum_bloat',
    ruleName: 'Table Bloat & Autovacuum Lag',
    clusterId: 'c1a7001-pg16-prod',
    clusterName: 'prod-pg16-ha-primary',
    severity: 'warning',
    summary: 'Table orders_partition_2026 has 18.4% dead tuples (1.4 GB bloat)',
    recommendation: 'Run VACUUM ANALYZE orders_partition_2026 and lower autovacuum_vacuum_cost_limit.',
    detectedAt: '25 mins ago',
    status: 'ACTIVE'
  },
  {
    id: 'fnd_02_conn',
    ruleId: 'rule_connection_ceiling',
    ruleName: 'High Connection Saturation (>80%)',
    clusterId: 'c2b8002-my84-prod',
    clusterName: 'core-mysql-ecommerce',
    severity: 'critical',
    summary: 'Active connections reached 412 / 500 (82.4% capacity)',
    recommendation: 'Deploy ProxySQL connection pooler or increase max_connections after checking RAM headroom.',
    detectedAt: '12 mins ago',
    status: 'ACTIVE'
  },
  {
    id: 'fnd_03_tls',
    ruleId: 'rule_non_tls_listener',
    ruleName: 'Unencrypted Plaintext Database Traffic',
    clusterId: 'c3c9003-ma11-galera',
    clusterName: 'bi-reporting-galera',
    severity: 'critical',
    summary: 'Galera node-ma-02 accepting non-SSL TCP client connections on port 3306',
    recommendation: 'Enable require_secure_transport=ON and distribute mTLS certificates via Nodlance CA.',
    detectedAt: '1 hour ago',
    status: 'ACTIVE'
  }
];

export const INITIAL_SLOW_QUERIES = [
  {
    queryHash: 'q_hash_e8912',
    clusterId: 'c1a7001-pg16-prod',
    clusterName: 'prod-pg16-ha-primary',
    databaseName: 'nodlance_prod',
    digestText: 'SELECT o.id, o.customer_id, SUM(i.price * i.qty) FROM orders o JOIN order_items i ON o.id = i.order_id WHERE o.created_at >= $1 GROUP BY o.id, o.customer_id ORDER BY o.created_at DESC LIMIT $2',
    executionCount: 145020,
    meanLatencyMs: 342.5,
    p95LatencyMs: 820.0,
    maxLatencyMs: 2410.0,
    totalLatencyMs: 49669350.0,
    rowsExamined: 12500000,
    rowsSent: 145020,
    lastSeen: 'Just now'
  },
  {
    queryHash: 'q_hash_a1194',
    clusterId: 'c2b8002-my84-prod',
    clusterName: 'core-mysql-ecommerce',
    databaseName: 'ecommerce_catalog',
    digestText: 'SELECT * FROM products p WHERE p.category_id = ? AND p.is_active = 1 AND p.stock > 0 ORDER BY p.sales_rank DESC',
    executionCount: 981200,
    meanLatencyMs: 185.2,
    p95LatencyMs: 410.0,
    maxLatencyMs: 1250.0,
    totalLatencyMs: 181718240.0,
    rowsExamined: 45000000,
    rowsSent: 981200,
    lastSeen: '1m ago'
  },
  {
    queryHash: 'q_hash_33b82',
    clusterId: 'c3c9003-ma11-galera',
    clusterName: 'bi-reporting-galera',
    databaseName: 'bi_warehouse',
    digestText: 'SELECT date_trunc(\'month\', event_time) AS m, user_segment, COUNT(DISTINCT user_id) FROM analytics_events WHERE event_time >= ? GROUP BY 1, 2',
    executionCount: 320,
    meanLatencyMs: 2840.0,
    p95LatencyMs: 6100.0,
    maxLatencyMs: 11400.0,
    totalLatencyMs: 908800.0,
    rowsExamined: 82000000,
    rowsSent: 320,
    lastSeen: '8m ago'
  }
];

export const INITIAL_CONFIG_TEMPLATES = [
  {
    id: 'tmpl_pg16_prod',
    name: 'PostgreSQL 16 High-Throughput Baseline',
    engine: 'postgres',
    clusterId: 'c1a7001-pg16-prod',
    clusterName: 'prod-pg16-ha-primary',
    filePath: '/var/lib/postgresql/data/postgresql.conf',
    version: 4,
    updatedAt: '1 hour ago',
    content: `# PostgreSQL 16 HA Production Config
listen_addresses = '*'
port = 5432
max_connections = 300
shared_buffers = 16GB
effective_cache_size = 48GB
maintenance_work_mem = 2GB
work_mem = 64MB
min_wal_size = 2GB
max_wal_size = 16GB
checkpoint_completion_target = 0.9
checkpoint_timeout = 15min
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
wal_level = replica
max_wal_senders = 10
max_replication_slots = 10
archive_mode = on
archive_command = 'nodlance-agent backup wal-push %p'
ssl = on
ssl_cert_file = '/etc/ssl/nodlance/server.crt'
ssl_key_file = '/etc/ssl/nodlance/server.key'
ssl_ca_file = '/etc/ssl/nodlance/ca.crt'`
  },
  {
    id: 'tmpl_mysql84_prod',
    name: 'MySQL 8.4 InnoDB Enterprise Template',
    engine: 'mysql',
    clusterId: 'c2b8002-my84-prod',
    clusterName: 'core-mysql-ecommerce',
    filePath: '/etc/mysql/mysql.conf.d/mysqld.cnf',
    version: 2,
    updatedAt: '3 hours ago',
    content: `# MySQL 8.4 Enterprise Config
[mysqld]
bind-address = 0.0.0.0
port = 3306
server_id = 101
log_bin = mysql-bin
binlog_format = ROW
gtid_mode = ON
enforce_gtid_consistency = ON
max_connections = 500
innodb_buffer_pool_size = 24GB
innodb_buffer_pool_instances = 8
innodb_log_file_size = 2GB
innodb_flush_log_at_trx_commit = 1
innodb_flush_method = O_DIRECT
require_secure_transport = ON
ssl_ca = /etc/mysql/nodlance-pki/ca.crt
ssl_cert = /etc/mysql/nodlance-pki/server.crt
ssl_key = /etc/mysql/nodlance-pki/server.key`
  },
  {
    id: 'tmpl_galera_ma11',
    name: 'MariaDB 11 Galera Multi-Master Config',
    engine: 'galera',
    clusterId: 'c3c9003-ma11-galera',
    clusterName: 'bi-reporting-galera',
    filePath: '/etc/mysql/mariadb.conf.d/60-galera.cnf',
    version: 3,
    updatedAt: 'Yesterday',
    content: `# MariaDB Galera Cluster Configuration
[mysqld]
binlog_format = ROW
default_storage_engine = InnoDB
innodb_autoinc_lock_mode = 2
bind-address = 0.0.0.0
port = 3306

[galera]
wsrep_on = ON
wsrep_provider = /usr/lib/galera/libgalera_smm.so
wsrep_cluster_name = "nodlance_galera_prod"
wsrep_cluster_address = "gcomm://192.168.10.41,192.168.10.42,192.168.10.43"
wsrep_sst_method = mariabackup
wsrep_node_name = "node-ma-01"
wsrep_node_address = "192.168.10.41"
wsrep_provider_options = "socket.ssl_key=/etc/ssl/nodlance/server.key;socket.ssl_cert=/etc/ssl/nodlance/server.crt;socket.ssl_ca=/etc/ssl/nodlance/ca.crt"`
  }
];

export const INITIAL_PKI_CERTIFICATES = [
  {
    id: 'cert_root_ca',
    clusterId: null,
    clusterName: 'Global Control Plane',
    commonName: 'Nodlance Root CA (Self-Signed Authority)',
    certType: 'root_ca',
    fingerprintSha256: '9f83a21b8c04918e77a23cba99120e047783cb2d110825f828a8d447e12365a1',
    validFrom: '2026-01-01',
    expiresAt: '2036-01-01',
    daysLeft: 3350,
    isRevoked: false,
    status: 'ACTIVE'
  },
  {
    id: 'cert_pg_node_01',
    clusterId: 'c1a7001-pg16-prod',
    clusterName: 'prod-pg16-ha-primary',
    commonName: 'db-pg-node-01.prod.internal',
    certType: 'server',
    fingerprintSha256: 'c341ba228a64cf9110bce904772154dd9841ca2b98e722891395fa33924bb312',
    validFrom: '2026-03-01',
    expiresAt: '2027-03-01',
    daysLeft: 312,
    isRevoked: false,
    status: 'ACTIVE'
  },
  {
    id: 'cert_pg_node_02',
    clusterId: 'c1a7001-pg16-prod',
    clusterName: 'prod-pg16-ha-primary',
    commonName: 'db-pg-node-02.prod.internal',
    certType: 'server',
    fingerprintSha256: '88de1a0293ffbca10928374ba81100223947ca6110ef444102948bbcd9012354',
    validFrom: '2026-03-01',
    expiresAt: '2027-03-01',
    daysLeft: 312,
    isRevoked: false,
    status: 'ACTIVE'
  },
  {
    id: 'cert_pg_node_03',
    clusterId: 'c1a7001-pg16-prod',
    clusterName: 'prod-pg16-ha-primary',
    commonName: 'db-pg-node-03.prod.internal',
    certType: 'server',
    fingerprintSha256: '1a0293ffbca10928374ba81100223947ca6110ef444102948bbcd901235488de',
    validFrom: '2026-03-01',
    expiresAt: '2027-03-01',
    daysLeft: 312,
    isRevoked: false,
    status: 'ACTIVE'
  },
  {
    id: 'cert_pgb_pooler',
    clusterId: 'c1a7001-pg16-prod',
    clusterName: 'prod-pg16-ha-primary',
    commonName: 'pgbouncer-pooler.prod.internal',
    certType: 'client',
    fingerprintSha256: '44a8bbcd901235488de1a0293ffbca10928374ba81100223947ca6110ef44410',
    validFrom: '2026-03-10',
    expiresAt: '2027-03-10',
    daysLeft: 321,
    isRevoked: false,
    status: 'ACTIVE'
  },
  {
    id: 'cert_galera_node_01',
    clusterId: 'c3c9003-ma11-galera',
    clusterName: 'bi-reporting-galera',
    commonName: 'galera-node-01.internal',
    certType: 'server',
    fingerprintSha256: 'ee904772154dd9841ca2b98e722891395fa33924bb312c341ba228a64cf9110b',
    validFrom: '2026-02-15',
    expiresAt: '2027-02-15',
    daysLeft: 298,
    isRevoked: false,
    status: 'ACTIVE'
  }
];



