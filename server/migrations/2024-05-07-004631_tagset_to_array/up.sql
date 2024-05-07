-- Your SQL goes here
ALTER TABLE chunk_metadata ADD COLUMN tag_set_array text[];

UPDATE chunk_metadata
SET tag_set_array = array_remove(string_to_array(tag_set, ','), 'null');

ALTER TABLE chunk_metadata DROP COLUMN tag_set;

ALTER TABLE chunk_metadata RENAME COLUMN tag_set_array TO tag_set;

ALTER TABLE files ADD COLUMN tag_set_array text[];

UPDATE files
SET tag_set_array = array_remove(string_to_array(tag_set, ','), 'null');

ALTER TABLE files DROP COLUMN tag_set;

ALTER TABLE files RENAME COLUMN tag_set_array TO tag_set;

ALTER TABLE chunk_group ADD COLUMN tag_set_array text[];

UPDATE chunk_group
SET tag_set_array = array_remove(string_to_array(tag_set, ','), 'null');

ALTER TABLE chunk_group DROP COLUMN tag_set;

ALTER TABLE chunk_group RENAME COLUMN tag_set_array TO tag_set;