-- This file should undo anything in `up.sql`
ALTER TABLE users
DROP COLUMN username,
DROP COLUMN visible_email;
