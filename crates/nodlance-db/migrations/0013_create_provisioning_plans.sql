-- Migration 0013: Create provisioning plans and step tracking tables

CREATE TABLE IF NOT EXISTS provisioning_plans (
    id UUID PRIMARY KEY,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
    host_id UUID REFERENCES hosts(id) ON DELETE SET NULL,
    provider_type TEXT NOT NULL,
    engine TEXT NOT NULL,
    engine_version TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_provisioning_plans_cluster_id ON provisioning_plans(cluster_id);
CREATE INDEX IF NOT EXISTS idx_provisioning_plans_status ON provisioning_plans(status);

CREATE TABLE IF NOT EXISTS provisioning_steps (
    id UUID PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES provisioning_plans(id) ON DELETE CASCADE,
    step_index INTEGER NOT NULL,
    step_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    error_details TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    CONSTRAINT uq_plan_step_index UNIQUE(plan_id, step_index)
);

CREATE INDEX IF NOT EXISTS idx_provisioning_steps_plan_id ON provisioning_steps(plan_id);
