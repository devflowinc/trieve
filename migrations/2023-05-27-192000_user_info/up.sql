-- Your SQL goes here
ALTER TABLE users
ADD COLUMN username TEXT NULL UNIQUE,
ADD COLUMN website TEXT NULL,
ADD COLUMN visible_email BOOLEAN DEFAULT false NOT NULL;

-- Set visible_email to false for all existing rows
UPDATE users
SET visible_email = true;
