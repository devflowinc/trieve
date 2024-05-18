-- Your SQL goes here
DROP INDEX idx_card_metadata_link;
DROP INDEX idx_card_metadata_oc_file_path;
DROP INDEX idx_chunk_metadata_created_at;
DROP INDEX idx_chunk_metadata_time_stamp;
CREATE INDEX idx_chunk_metadata_json ON chunk_metadata USING gin (metadata jsonb_path_ops);

-- Update the tag_set column to be an array of text for chunk_metadata
ALTER TABLE chunk_metadata ADD COLUMN tag_set_array_column text[];

UPDATE chunk_metadata
SET tag_set_array_column = array_remove(string_to_array(tag_set, ','), NULL);

ALTER TABLE chunk_metadata DROP COLUMN tag_set;

ALTER TABLE chunk_metadata RENAME COLUMN tag_set_array_column TO tag_set;

-- Update the tag_set column to be an array of text for chunk_group
ALTER TABLE chunk_group ADD COLUMN tag_set_array_column text[];

UPDATE chunk_group
SET tag_set_array_column = array_remove(string_to_array(tag_set, ','), NULL);

ALTER TABLE chunk_group DROP COLUMN tag_set;

ALTER TABLE chunk_group RENAME COLUMN tag_set_array_column TO tag_set;

-- Update the tag_set column to be an array of text for files
ALTER TABLE files ADD COLUMN tag_set_array_column text[];

UPDATE files
SET tag_set_array_column = array_remove(string_to_array(tag_set, ','), NULL);

ALTER TABLE files DROP COLUMN tag_set;

ALTER TABLE files RENAME COLUMN tag_set_array_column TO tag_set;