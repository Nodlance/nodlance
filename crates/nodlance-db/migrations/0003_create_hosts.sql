-- 0003_create_hosts.sql
CREATE TABLE IF NOT EXISTS hosts (
    id UUID PRIMARY KEY,
    hostname TEXT NOT NULL,
    ip_address INET,
    os_family TEXT,
    os_version TEXT,
    labels JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'decommissioned')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_hosts_hostname ON hosts(hostname);
CREATE INDEX IF NOT EXISTS idx_hosts_status ON hosts(status);
CREATE INDEX IF NOT EXISTS idx_hosts_created_by ON hosts(created_by);
