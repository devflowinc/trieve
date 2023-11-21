-- Your SQL goes here
ALTER TABLE card_metadata ADD COLUMN time_stamp TIMESTAMP NOT NULL DEFAULT NOW();
