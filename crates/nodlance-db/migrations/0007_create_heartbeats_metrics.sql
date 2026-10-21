-- 0007_create_heartbeats_metrics.sql
CREATE TABLE IF NOT EXISTS agent_heartbeats (
    id UUID PRIMARY KEY,
    host_id UUID NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    agent_version TEXT NOT NULL,
    cpu_percent REAL,
    memory_used_bytes BIGINT,
    memory_total_bytes BIGINT,
    disk_used_bytes BIGINT,
    disk_total_bytes BIGINT,
    load_avg_one REAL
);

CREATE INDEX IF NOT EXISTS idx_agent_heartbeats_host_id ON agent_heartbeats(host_id);
CREATE INDEX IF NOT EXISTS idx_agent_heartbeats_timestamp ON agent_heartbeats(timestamp DESC);
