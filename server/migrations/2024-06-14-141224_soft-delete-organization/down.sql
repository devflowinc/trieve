-- This file should undo anything in `up.sql`
ALTER TABLE organizations DROP COLUMN IF EXISTS deleted;
DROP INDEX IF EXISTS idx_organization_deleted;