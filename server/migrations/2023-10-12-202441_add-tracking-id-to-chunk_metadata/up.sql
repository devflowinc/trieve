-- Your SQL goes here
ALTER TABLE chunk_metadata
ADD COLUMN tracking_id TEXT UNIQUE;
