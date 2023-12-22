-- This file should undo anything in `up.sql`
ALTER TABLE chunk_collisions
  DROP COLUMN created_at,
  DROP COLUMN updated_at;
