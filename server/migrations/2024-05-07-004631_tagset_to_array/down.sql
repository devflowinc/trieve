-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata DROP COLUMN tag_set;
ALTER TABLE chunk_metadata ADD COLUMN tag_set text;

ALTER TABLE files DROP COLUMN tag_set;
ALTER TABLE files ADD COLUMN tag_set text;

ALTER TABLE chunk_group DROP COLUMN tag_set;
ALTER TABLE chunk_group ADD COLUMN tag_set text;
