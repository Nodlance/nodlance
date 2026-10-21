-- 0012_create_restore_drills.sql
CREATE TABLE IF NOT EXISTS restore_validation_drills (
    id UUID PRIMARY KEY,
    backup_id UUID NOT NULL REFERENCES backup_artifacts(id) ON DELETE CASCADE,
    verified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_valid BOOLEAN NOT NULL,
    verification_report JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_restore_drills_backup_id ON restore_validation_drills(backup_id);
CREATE INDEX IF NOT EXISTS idx_restore_drills_verified_at ON restore_validation_drills(verified_at DESC);
