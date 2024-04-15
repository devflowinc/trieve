-- This file should undo anything in `up.sql`
ALTER TABLE datasets DROP COLUMN IF EXISTS tracking_id;
DROP INDEX IF EXISTS datasets_tracking_id_idx;