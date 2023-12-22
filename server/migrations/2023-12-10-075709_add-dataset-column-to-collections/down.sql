-- This file should undo anything in `up.sql`
ALTER TABLE
    chunk_metadata DROP COLUMN IF EXISTS dataset_id;

ALTER TABLE
    chunk_collection DROP COLUMN IF EXISTS dataset_id;