-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata DROP COLUMN location;
ALTER TABLE chunk_metadata ADD COLUMN location JSONB DEFAULT '{}' NULL;