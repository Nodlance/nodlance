-- 0006_create_clusters_nodes.sql
CREATE TABLE IF NOT EXISTS clusters (
    id UUID PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    engine TEXT NOT NULL CHECK (engine IN ('postgresql', 'mysql', 'mariadb')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS database_nodes (
    id UUID PRIMARY KEY,
    cluster_id UUID NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
    host_id UUID NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    port INT NOT NULL DEFAULT 5432,
    role TEXT NOT NULL DEFAULT 'unknown' CHECK (role IN ('primary', 'replica', 'standalone', 'unknown')),
    health TEXT NOT NULL DEFAULT 'unknown' CHECK (health IN ('healthy', 'degraded', 'unreachable', 'unknown')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(cluster_id, name)
);

CREATE INDEX IF NOT EXISTS idx_database_nodes_cluster_id ON database_nodes(cluster_id);
CREATE INDEX IF NOT EXISTS idx_database_nodes_host_id ON database_nodes(host_id);
