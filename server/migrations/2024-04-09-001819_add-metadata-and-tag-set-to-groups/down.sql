-- This file should undo anything in `up.sql`
ALTER TABLE chunk_group DROP COLUMN IF EXISTS metadata;
ALTER TABLE chunk_group DROP COLUMN IF EXISTS tag_set;

DROP INDEX IF EXISTS idx_chunk_group_metadata;
DROP INDEX IF EXISTS idx_chunk_group_tag_set;