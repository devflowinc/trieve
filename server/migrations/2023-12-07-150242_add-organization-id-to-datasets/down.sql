-- This file should undo anything in `up.sql`
ALTER TABLE
    datasets DROP COLUMN IF EXISTS organization_id;