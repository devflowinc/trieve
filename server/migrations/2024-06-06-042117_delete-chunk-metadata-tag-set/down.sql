-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata
ADD COLUMN tag_set TEXT[] DEFAULT '{}';
