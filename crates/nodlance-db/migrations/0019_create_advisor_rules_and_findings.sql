-- Migration 0019: Create advisor rules, findings, and slow query digest telemetry tables

CREATE TABLE IF NOT EXISTS advisor_rules (
    id VARCHAR(64) PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    engine VARCHAR(32) NOT NULL DEFAULT 'any',
    category VARCHAR(32) NOT NULL, -- 'performance', 'security', 'capacity', 'availability'
    severity VARCHAR(16) NOT NULL, -- 'info', 'warning', 'critical'
    description TEXT NOT NULL,
    script_content TEXT NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS advisor_findings (
    id UUID PRIMARY KEY,
    rule_id VARCHAR(64) NOT NULL REFERENCES advisor_rules(id) ON DELETE CASCADE,
    rule_name VARCHAR(128) NOT NULL,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
    node_id UUID REFERENCES database_nodes(id) ON DELETE CASCADE,
    severity VARCHAR(16) NOT NULL,
    summary TEXT NOT NULL,
    recommendation TEXT NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_advisor_findings_cluster_id ON advisor_findings(cluster_id);
CREATE INDEX IF NOT EXISTS idx_advisor_findings_rule_id ON advisor_findings(rule_id);
CREATE INDEX IF NOT EXISTS idx_advisor_findings_resolved_at ON advisor_findings(resolved_at);

CREATE TABLE IF NOT EXISTS query_digests (
    query_hash VARCHAR(64) NOT NULL,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
    node_id UUID REFERENCES database_nodes(id) ON DELETE CASCADE,
    database_name VARCHAR(64) NOT NULL DEFAULT 'public',
    digest_text TEXT NOT NULL,
    execution_count BIGINT NOT NULL DEFAULT 1,
    total_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    mean_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    p95_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    max_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    rows_examined BIGINT NOT NULL DEFAULT 0,
    rows_sent BIGINT NOT NULL DEFAULT 0,
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (cluster_id, query_hash)
);

CREATE INDEX IF NOT EXISTS idx_query_digests_cluster_id ON query_digests(cluster_id);
CREATE INDEX IF NOT EXISTS idx_query_digests_mean_latency ON query_digests(mean_latency_ms DESC);

-- Seed default out-of-the-box ClusterControl DBA advisor heuristics
INSERT INTO advisor_rules (id, name, engine, category, severity, description, script_content, is_enabled)
VALUES
('rule_innodb_buffer_pool', 'Buffer Pool Cache Hit Ratio Low', 'mysql', 'performance', 'warning',
 'InnoDB buffer pool hit ratio is below 95%. Consider increasing innodb_buffer_pool_size to reduce disk read IOPS.',
 'buffer_pool_hit_ratio < 0.95', TRUE),

('rule_connection_ceiling', 'High Connection Saturation (>80%)', 'any', 'capacity', 'critical',
 'Active connections exceed 80% of max_connections limit. Risk of connection starvation. Deploy PgBouncer or ProxySQL connection pooling.',
 '(active_connections * 100) / max_connections > 80', TRUE),

('rule_autovacuum_bloat', 'Table Bloat & Autovacuum Lag', 'postgres', 'capacity', 'warning',
 'Dead tuples ratio exceeds 15%. Autovacuum workers may be lagging. Check autovacuum_vacuum_scale_factor and autovacuum_max_workers.',
 'dead_tuples_ratio > 0.15', TRUE),

('rule_missing_primary_keys', 'Tables Without Primary Keys', 'any', 'availability', 'critical',
 'Discovered tables lacking primary keys. This breaks row-based replication consistency and Galera multi-primary replication certification.',
 'missing_primary_keys_count > 0', TRUE),

('rule_non_tls_listener', 'Unencrypted Plaintext Database Traffic', 'any', 'security', 'critical',
 'Cluster is operating without enforced TLS/SSL encryption. Database passwords and sensitive data are vulnerable to network snooping. Enable mTLS.',
 '!tls_enabled', TRUE),

('rule_galera_flow_control', 'Galera Cluster Flow Control Paused Stalling', 'galera', 'availability', 'critical',
 'wsrep_flow_control_paused exceeds 5%. One or more Galera nodes cannot keep up with write rate, causing replication pauses across the entire cluster.',
 'galera_flow_control_paused_pct > 5.0', TRUE),

('rule_replication_lag', 'Replica Lag Exceeds Operational SLA', 'any', 'availability', 'warning',
 'Replication lag to follower replicas exceeds 1000ms. Reads routed to replicas may return stale transactions. Investigate disk IOPS on replica nodes.',
 'replication_lag_ms > 1000', TRUE),

('rule_super_privilege_exposure', 'Untrusted Superuser Privilege Exposure', 'any', 'security', 'critical',
 'Non-localhost database accounts have been granted superuser/SUPER privileges. Limit administrative access to dedicated management bastions.',
 'super_privilege_non_localhost', TRUE)
ON CONFLICT (id) DO NOTHING;
