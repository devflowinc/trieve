-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata DROP COLUMN num_value;
DROP INDEX IF EXISTS idx_num_val_chunk_metadata;