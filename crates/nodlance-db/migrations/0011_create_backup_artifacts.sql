-- 0011_create_backup_artifacts.sql
CREATE TABLE IF NOT EXISTS backup_artifacts (
    id UUID PRIMARY KEY,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
    node_id UUID NOT NULL REFERENCES database_nodes(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    engine TEXT NOT NULL,
    engine_version TEXT NOT NULL,
    backup_type TEXT NOT NULL CHECK (backup_type IN ('full', 'incremental', 'differential')),
    storage_location TEXT NOT NULL,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    checksum_sha256 TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'completed', 'corrupt', 'failed')),
    retention_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS retention_policies (
    id UUID PRIMARY KEY,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
    retention_days INT NOT NULL DEFAULT 30,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(cluster_id)
);

CREATE INDEX IF NOT EXISTS idx_backup_artifacts_cluster_id ON backup_artifacts(cluster_id);
CREATE INDEX IF NOT EXISTS idx_backup_artifacts_node_id ON backup_artifacts(node_id);
CREATE INDEX IF NOT EXISTS idx_backup_artifacts_retention_until ON backup_artifacts(retention_until);
