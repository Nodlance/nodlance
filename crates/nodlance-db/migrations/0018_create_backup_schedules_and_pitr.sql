-- Migration 0018: Create backup schedules and PITR continuous stream catalogues

CREATE TABLE IF NOT EXISTS backup_schedules (
    id UUID PRIMARY KEY,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    cron_expression TEXT NOT NULL,
    backup_method TEXT NOT NULL,
    storage_target TEXT NOT NULL,
    retention_days INT NOT NULL DEFAULT 30,
    compression TEXT NOT NULL DEFAULT 'zstd',
    encrypt BOOLEAN NOT NULL DEFAULT true,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_backup_schedules_cluster_id ON backup_schedules(cluster_id);
CREATE INDEX IF NOT EXISTS idx_backup_schedules_is_active ON backup_schedules(is_active);

CREATE TABLE IF NOT EXISTS pitr_streams (
    id UUID PRIMARY KEY,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE UNIQUE,
    storage_path TEXT NOT NULL,
    start_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    latest_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    start_lsn_or_gtid TEXT NOT NULL DEFAULT '',
    latest_lsn_or_gtid TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'active',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pitr_streams_cluster_id ON pitr_streams(cluster_id);
CREATE INDEX IF NOT EXISTS idx_pitr_streams_status ON pitr_streams(status);
