-- Your SQL goes here
ALTER TABLE card_metadata DROP COLUMN IF EXISTS card_metadata_tsvector;
DROP TABLE IF EXISTS invitations;
DROP TABLE IF EXISTS password_resets;
