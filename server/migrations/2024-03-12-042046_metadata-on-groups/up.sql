-- Your SQL goes here
ALTER TABLE chunk_group ADD COLUMN metadata JSONB DEFAULT '{}'::jsonb;