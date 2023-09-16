-- This file should undo anything in `up.sql`
ALTER TABLE files
  DROP COLUMN size;
