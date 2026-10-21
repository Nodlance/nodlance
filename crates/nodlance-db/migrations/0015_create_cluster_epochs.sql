-- Migration 0015: Create cluster epochs tracking table for term/epoch monotonicity

CREATE TABLE IF NOT EXISTS cluster_epochs (
    id UUID PRIMARY KEY,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
    epoch BIGINT NOT NULL,
    promoted_host_id UUID REFERENCES hosts(id) ON DELETE SET NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_cluster_epoch UNIQUE(cluster_id, epoch)
);

CREATE INDEX IF NOT EXISTS idx_cluster_epochs_cluster_id ON cluster_epochs(cluster_id);
CREATE INDEX IF NOT EXISTS idx_cluster_epochs_epoch ON cluster_epochs(cluster_id, epoch DESC);
