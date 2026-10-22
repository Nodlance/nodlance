-- Migration 0020: Create configuration templates and PKI certificates tables

CREATE TABLE IF NOT EXISTS config_templates (
    id UUID PRIMARY KEY,
    cluster_id UUID REFERENCES clusters(id) ON DELETE CASCADE,
    name VARCHAR(128) NOT NULL,
    engine VARCHAR(32) NOT NULL,
    file_path VARCHAR(256) NOT NULL,
    content TEXT NOT NULL,
    version INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_config_templates_cluster_id ON config_templates(cluster_id);
CREATE INDEX IF NOT EXISTS idx_config_templates_engine ON config_templates(engine);

CREATE TABLE IF NOT EXISTS pki_certificates (
    id UUID PRIMARY KEY,
    cluster_id UUID REFERENCES clusters(id) ON DELETE CASCADE,
    common_name VARCHAR(128) NOT NULL,
    cert_type VARCHAR(32) NOT NULL, -- 'root_ca', 'server', 'client'
    certificate_pem TEXT NOT NULL,
    fingerprint_sha256 VARCHAR(64) NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pki_certificates_cluster_id ON pki_certificates(cluster_id);
CREATE INDEX IF NOT EXISTS idx_pki_certificates_cert_type ON pki_certificates(cert_type);
CREATE INDEX IF NOT EXISTS idx_pki_certificates_expires_at ON pki_certificates(expires_at);

-- Pre-seed default production baseline templates
INSERT INTO config_templates (id, cluster_id, name, engine, file_path, content, version, created_at, updated_at)
VALUES
(
    '018f0000-0000-7000-8000-000000000001'::uuid,
    NULL,
    'PostgreSQL 16 High-Throughput Baseline',
    'postgres',
    '/etc/postgresql/16/main/postgresql.conf',
    E'# Nodlance Optimized PostgreSQL 16 Baseline\nmax_connections = 300\nshared_buffers = 8GB\neffective_cache_size = 24GB\nmaintenance_work_mem = 2GB\ncheckpoint_completion_target = 0.9\nwal_buffers = 64MB\ndefault_statistics_target = 100\nrandom_page_cost = 1.1\neffective_io_concurrency = 200\nwork_mem = 32MB\nmin_wal_size = 2GB\nmax_wal_size = 16GB\nmax_worker_processes = 8\nmax_parallel_workers_per_gather = 4\nmax_parallel_workers = 8\nwal_level = replica\narchive_mode = on\nssl = on\n',
    1,
    NOW(),
    NOW()
),
(
    '018f0000-0000-7000-8000-000000000002'::uuid,
    NULL,
    'MySQL 8.4 High Concurrency Baseline',
    'mysql',
    '/etc/mysql/my.cnf',
    E'# Nodlance Optimized MySQL 8.4 Baseline\n[mysqld]\nmax_connections = 500\ninnodb_buffer_pool_size = 12G\ninnodb_log_file_size = 2G\ninnodb_flush_log_at_trx_commit = 1\ninnodb_flush_method = O_DIRECT\ninnodb_file_per_table = 1\ninnodb_open_files = 4000\nopen_files_limit = 65535\nmax_allowed_packet = 64M\nslow_query_log = 1\nlong_query_time = 0.5\nrequire_secure_transport = ON\n',
    1,
    NOW(),
    NOW()
)
ON CONFLICT (id) DO NOTHING;
