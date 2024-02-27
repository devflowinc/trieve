-- This file should undo anything in `up.sql`
ALTER TABLE chunk_group_bookmarks
DROP CONSTRAINT group_id_chunk_metadata_id_key;