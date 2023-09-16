-- Your SQL goes here
ALTER TABLE files
  ADD COLUMN size bigint NOT NULL DEFAULT 0;
