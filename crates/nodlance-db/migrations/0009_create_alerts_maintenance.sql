-- 0009_create_alerts_maintenance.sql
CREATE TABLE IF NOT EXISTS alert_rules (
    id UUID PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    condition TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'critical')),
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS alert_incidents (
    id UUID PRIMARY KEY,
    rule_id UUID REFERENCES alert_rules(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'critical')),
    status TEXT NOT NULL DEFAULT 'firing' CHECK (status IN ('firing', 'acknowledged', 'resolved')),
    resource_type TEXT NOT NULL,
    resource_id UUID,
    fired_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS maintenance_windows (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    resource_type TEXT,
    resource_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_alert_incidents_status ON alert_incidents(status);
CREATE INDEX IF NOT EXISTS idx_alert_incidents_fired_at ON alert_incidents(fired_at DESC);
CREATE INDEX IF NOT EXISTS idx_maintenance_windows_range ON maintenance_windows(starts_at, ends_at);

-- Seed default alert rules
INSERT INTO alert_rules (id, name, condition, severity, is_enabled) VALUES
    ('0192e2c0-0001-7000-8000-000000000001', 'agent_heartbeat_timeout', 'heartbeat_delay > 45s', 'critical', true),
    ('0192e2c0-0002-7000-8000-000000000002', 'database_node_unreachable', 'health == unreachable', 'critical', true),
    ('0192e2c0-0003-7000-8000-000000000003', 'replication_lag_high', 'lag > 30s', 'warning', true)
ON CONFLICT (name) DO NOTHING;
