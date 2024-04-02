-- This file should undo anything in `up.sql`
ALTER TABLE users
  ADD COLUMN username TEXT;

ALTER TABLE users
  ADD COLUMN visible_email BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE users
  ADD COLUMN website TEXT;
