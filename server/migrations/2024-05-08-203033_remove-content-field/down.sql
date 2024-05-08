-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata ADD COLUMN content text;
