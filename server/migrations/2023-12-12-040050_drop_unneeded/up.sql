-- Your SQL goes here
ALTER TABLE card_metadata DROP COLUMN IF EXISTS card_metadata_tsvector;
DROP TABLE IF EXISTS invitations;
DROP TABLE IF EXISTS password_resets;
DROP TRIGGER IF EXISTS update_tsvector_trigger ON card_metadata;
DROP FUNCTION IF EXISTS update_tsvector();
