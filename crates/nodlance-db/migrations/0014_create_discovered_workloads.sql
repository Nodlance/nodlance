-- Migration 0014: Create discovered workloads table

CREATE TABLE IF NOT EXISTS discovered_workloads (
    id UUID PRIMARY KEY,
    provider_type TEXT NOT NULL,
    namespace TEXT,
    workload_name TEXT NOT NULL,
    engine TEXT NOT NULL,
    detected_version TEXT,
    endpoints JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL,
    details JSONB,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_discovered_workloads_provider ON discovered_workloads(provider_type);
CREATE INDEX IF NOT EXISTS idx_discovered_workloads_engine ON discovered_workloads(engine);
CREATE UNIQUE INDEX IF NOT EXISTS uq_discovered_workload ON discovered_workloads(provider_type, COALESCE(namespace, ''), workload_name);
