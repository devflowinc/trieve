-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata DROP COLUMN tag_set_array;
ALTER TABLE chunk_group DROP COLUMN tag_set_array;
ALTER TABLE files DROP COLUMN tag_set_array;