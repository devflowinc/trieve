-- Your SQL goes here
ALTER TABLE chunk_metadata
RENAME COLUMN oc_file_path TO tag_set;

ALTER TABLE chunk_metadata
ADD COLUMN metadata JSONB NULL DEFAULT '{}'::JSONB;

ALTER TABLE files
RENAME COLUMN oc_file_path TO tag_set;
