-- Your SQL goes here
ALTER TABLE users
ADD COLUMN username TEXT NULL,
ADD COLUMN visible_email BOOLEAN NULL DEFAULT false;
