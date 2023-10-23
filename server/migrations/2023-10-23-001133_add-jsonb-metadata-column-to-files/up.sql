-- Your SQL goes here
ALTER TABLE files
ADD COLUMN metadata JSONB NULL DEFAULT '{}'::JSONB;

ALTER TABLE files
DROP COLUMN mime_type;