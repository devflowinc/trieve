-- Your SQL goes here
ALTER TABLE card_metadata
ADD COLUMN dataset TEXT NOT NULL default 'DEFAULT';
