-- Your SQL goes here
ALTER TABLE card_metadata
RENAME COLUMN link TO filter_one;

ALTER TABLE card_metadata
RENAME COLUMN oc_file_path TO filter_two;

ALTER TABLE card_metadata
ADD COLUMN metadata JSONB NULL DEFAULT '{}'::JSONB;

ALTER TABLE files
RENAME COLUMN oc_file_path TO filter_two;
