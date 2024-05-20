-- This file should undo anything in `up.sql`
DROP INDEX idx_chunk_metadata_json;
CREATE INDEX idx_card_metadata_link ON card_metadata (link);
CREATE INDEX idx_card_metadata_oc_file_path ON card_metadata (oc_file_path);
CREATE INDEX idx_chunk_metadata_created_at ON chunk_metadata (created_at);
CREATE INDEX idx_chunk_metadata_time_stamp ON chunk_metadata (time_stamp);

-- Update the tag_set column to be a comma-separated string for chunk_metadata
ALTER TABLE chunk_metadata ADD COLUMN tag_set_string_column text;

UPDATE chunk_metadata
SET tag_set_string_column = array_to_string(tag_set, ',');

ALTER TABLE chunk_metadata DROP COLUMN tag_set;

ALTER TABLE chunk_metadata RENAME COLUMN tag_set_string_column TO tag_set;

-- Update the tag_set column to be a comma-separated string for chunk_group
ALTER TABLE chunk_group ADD COLUMN tag_set_string_column text;

UPDATE chunk_group
SET tag_set_string_column = array_to_string(tag_set, ',');

ALTER TABLE chunk_group DROP COLUMN tag_set;

ALTER TABLE chunk_group RENAME COLUMN tag_set_string_column TO tag_set;

-- Update the tag_set column to be a comma-separated string for files
ALTER TABLE files ADD COLUMN tag_set_string_column text;

UPDATE files
SET tag_set_string_column = array_to_string(tag_set, ',');

ALTER TABLE files DROP COLUMN tag_set;

ALTER TABLE files RENAME COLUMN tag_set_string_column TO tag_set;

