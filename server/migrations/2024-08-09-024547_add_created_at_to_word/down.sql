-- This file should undo anything in `up.sql`
ALTER TABLE words_datasets DROP COLUMN IF EXISTS created_at;