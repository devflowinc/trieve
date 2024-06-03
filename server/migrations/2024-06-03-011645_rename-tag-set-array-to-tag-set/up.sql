-- Your SQL goes here
ALTER TABLE chunk_metadata DROP COLUMN tag_set;

ALTER TABLE chunk_metadata RENAME COLUMN tag_set_array TO tag_set;

ALTER TABLE files DROP COLUMN tag_set;

ALTER TABLE files RENAME COLUMN tag_set_array TO tag_set;

ALTER TABLE chunk_group DROP COLUMN tag_set;

ALTER TABLE chunk_group RENAME COLUMN tag_set_array TO tag_set;