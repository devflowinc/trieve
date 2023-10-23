-- This file should undo anything in `up.sql`
ALTER TABLE files
DROP COLUMN metadata;

ALTER TABLE files
ADD COLUMN mime_type TEXT NULL DEFAULT NULL;
