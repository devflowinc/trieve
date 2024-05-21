-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata ADD COLUMN tag_set_array TEXT[] DEFAULT '{}';
ALTER TABLE chunk_group ADD COLUMN tag_set_array TEXT[] DEFAULT '{}';
ALTER TABLE files ADD COLUMN tag_set_array TEXT[] DEFAULT '{}';