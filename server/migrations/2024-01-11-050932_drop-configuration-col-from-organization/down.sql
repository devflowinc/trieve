-- This file should undo anything in `up.sql`
ALTER TABLE organizations ADD COLUMN configuration jsonb NOT NULL DEFAULT;