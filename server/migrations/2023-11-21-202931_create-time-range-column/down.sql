-- This file should undo anything in `up.sql`
ALTER TABLE card_metadata
DROP COLUMN IF EXISTS time_stamp;
ALTER TABLE files
DROP COLUMN IF EXISTS time_stamp;
