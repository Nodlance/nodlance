-- 0004_create_agent_identities.sql
CREATE TABLE IF NOT EXISTS agent_identities (
    id UUID PRIMARY KEY,
    host_id UUID NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    fingerprint TEXT UNIQUE NOT NULL,
    status TEXT NOT NULL DEFAULT 'enrolled' CHECK (status IN ('pending', 'enrolled', 'revoked')),
    enrolled_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    revoked_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_agent_identities_host_id ON agent_identities(host_id);
CREATE INDEX IF NOT EXISTS idx_agent_identities_fingerprint ON agent_identities(fingerprint);
