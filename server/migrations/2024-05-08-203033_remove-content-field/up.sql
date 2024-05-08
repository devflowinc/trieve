-- Your SQL goes here
ALTER TABLE chunk_metadata DROP COLUMN content;
ALTER TABLE chunk_metadata ALTER COLUMN chunk_html SET NOT NULL;
