-- This file should undo anything in `up.sql`
ALTER TABLE datasets DROP COLUMN IF EXISTS deleted;
DROP INDEX IF EXISTS idx_dataset_deleted;