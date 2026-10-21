-- 0008_create_observations.sql
CREATE TABLE IF NOT EXISTS node_observations (
    id UUID PRIMARY KEY,
    node_id UUID NOT NULL REFERENCES database_nodes(id) ON DELETE CASCADE,
    observed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    role TEXT NOT NULL,
    health TEXT NOT NULL,
    replication_lag_seconds DOUBLE PRECISION,
    is_read_only BOOLEAN NOT NULL DEFAULT false,
    details JSONB
);

CREATE INDEX IF NOT EXISTS idx_node_observations_node_id ON node_observations(node_id);
CREATE INDEX IF NOT EXISTS idx_node_observations_observed_at ON node_observations(observed_at DESC);
