-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata RENAME COLUMN tag_set TO tag_set_array;

ALTER TABLE chunk_metadata ADD COLUMN tag_set TEXT DEFAULT NULL;

ALTER TABLE files RENAME COLUMN tag_set TO tag_set_array;

ALTER TABLE files ADD COLUMN tag_set TEXT DEFAULT NULL;

ALTER TABLE chunk_group RENAME COLUMN tag_set TO tag_set_array;

ALTER TABLE chunk_group ADD COLUMN tag_set TEXT DEFAULT NULL;
