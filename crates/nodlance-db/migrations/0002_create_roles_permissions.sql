-- 0002_create_roles_permissions.sql
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT
);

CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

-- Seed permissions
INSERT INTO permissions (id, name, description) VALUES
    ('0192e2b0-0001-7000-8000-000000000001', 'user:read', 'View users and role assignments'),
    ('0192e2b0-0002-7000-8000-000000000002', 'user:write', 'Create and modify users and roles'),
    ('0192e2b0-0003-7000-8000-000000000003', 'host:read', 'View host inventory'),
    ('0192e2b0-0004-7000-8000-000000000004', 'host:write', 'Register, update, and decommission hosts'),
    ('0192e2b0-0005-7000-8000-000000000005', 'agent:write', 'Enroll and revoke agent identities'),
    ('0192e2b0-0006-7000-8000-000000000006', 'audit:read', 'Query audit event log')
ON CONFLICT (name) DO NOTHING;

-- Seed roles
INSERT INTO roles (id, name, description) VALUES
    ('0192e2b0-0010-7000-8000-000000000010', 'admin', 'Full control-plane administrative privileges'),
    ('0192e2b0-0020-7000-8000-000000000020', 'operator', 'Host and agent management without user administration'),
    ('0192e2b0-0030-7000-8000-000000000030', 'viewer', 'Read-only access to inventory and audit events')
ON CONFLICT (name) DO NOTHING;

-- Seed role_permissions for admin (all permissions)
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin'
ON CONFLICT DO NOTHING;

-- Seed role_permissions for operator (host:read, host:write, agent:write, audit:read)
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'operator' AND p.name IN ('host:read', 'host:write', 'agent:write', 'audit:read')
ON CONFLICT DO NOTHING;

-- Seed role_permissions for viewer (host:read, audit:read)
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'viewer' AND p.name IN ('host:read', 'audit:read')
ON CONFLICT DO NOTHING;
