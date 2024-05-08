-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata ADD COLUMN content text;
ALTER TABLE chunk_metadata ALTER COLUMN content DROP NOT NULL;