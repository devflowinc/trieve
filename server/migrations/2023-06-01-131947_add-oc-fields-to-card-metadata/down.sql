-- This file should undo anything in `up.sql`
ALTER TABLE card_metadata
DROP COLUMN oc_file_path;
