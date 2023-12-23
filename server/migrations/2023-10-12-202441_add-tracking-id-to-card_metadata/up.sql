-- Your SQL goes here
ALTER TABLE card_metadata
ADD COLUMN tracking_id TEXT UNIQUE;
