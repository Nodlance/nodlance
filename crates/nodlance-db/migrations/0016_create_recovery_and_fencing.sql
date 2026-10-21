-- Migration 0016: Create fencing executions and promotion requests tables

CREATE TABLE IF NOT EXISTS fencing_executions (
    id UUID PRIMARY KEY,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
    target_host_id UUID NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    fencing_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    verification_proof TEXT,
    error_message TEXT,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_fencing_executions_cluster_id ON fencing_executions(cluster_id);
CREATE INDEX IF NOT EXISTS idx_fencing_executions_target_host_id ON fencing_executions(target_host_id);
CREATE INDEX IF NOT EXISTS idx_fencing_executions_status ON fencing_executions(status);

CREATE TABLE IF NOT EXISTS promotion_requests (
    id UUID PRIMARY KEY,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
    candidate_host_id UUID NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    target_epoch BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending_approval',
    approval_token TEXT,
    approved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at TIMESTAMPTZ,
    fencing_execution_id UUID REFERENCES fencing_executions(id) ON DELETE SET NULL,
    safety_gates_passed BOOLEAN NOT NULL DEFAULT FALSE,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_promotion_requests_cluster_id ON promotion_requests(cluster_id);
CREATE INDEX IF NOT EXISTS idx_promotion_requests_status ON promotion_requests(status);
