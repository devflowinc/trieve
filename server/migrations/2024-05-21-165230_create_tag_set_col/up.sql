-- Your SQL goes here
ALTER TABLE chunk_metadata ADD COLUMN tag_set_array TEXT[] DEFAULT '{}';
ALTER TABLE chunk_group ADD COLUMN tag_set_array TEXT[] DEFAULT '{}';
ALTER TABLE files ADD COLUMN tag_set_array TEXT[] DEFAULT '{}';