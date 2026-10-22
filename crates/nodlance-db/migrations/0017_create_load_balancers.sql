-- Migration 0017: Create load balancers table for ProxySQL, PgBouncer, HAProxy, and Keepalived VIP

CREATE TABLE IF NOT EXISTS load_balancers (
    id UUID PRIMARY KEY,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
    host_id UUID NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    lb_type TEXT NOT NULL,
    listen_port INT NOT NULL,
    admin_port INT,
    status TEXT NOT NULL DEFAULT 'active',
    config_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_load_balancers_cluster_id ON load_balancers(cluster_id);
CREATE INDEX IF NOT EXISTS idx_load_balancers_host_id ON load_balancers(host_id);
CREATE INDEX IF NOT EXISTS idx_load_balancers_type ON load_balancers(lb_type);
CREATE INDEX IF NOT EXISTS idx_load_balancers_status ON load_balancers(status);
