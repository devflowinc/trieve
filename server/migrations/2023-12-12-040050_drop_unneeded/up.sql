-- Your SQL goes here
ALTER TABLE chunk_metadata DROP COLUMN IF EXISTS chunk_metadata_tsvector;
DROP TABLE IF EXISTS invitations;
DROP TABLE IF EXISTS password_resets;
DROP TRIGGER IF EXISTS update_tsvector_trigger ON chunk_metadata;
DROP FUNCTION IF EXISTS update_tsvector();
