-- Your SQL goes here
ALTER TABLE users DROP COLUMN hash;
ALTER TABLE users ADD COLUMN name TEXT NULL;
