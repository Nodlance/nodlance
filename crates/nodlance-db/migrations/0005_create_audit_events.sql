-- 0005_create_audit_events.sql
CREATE TABLE IF NOT EXISTS audit_events (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor_id UUID NOT NULL REFERENCES users(id),
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id UUID,
    details JSONB,
    source_ip INET
);

CREATE INDEX IF NOT EXISTS idx_audit_events_timestamp ON audit_events(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_events_actor_id ON audit_events(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_events_resource ON audit_events(resource_type, resource_id);
