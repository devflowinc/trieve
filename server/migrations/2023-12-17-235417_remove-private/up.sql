-- Your SQL goes here
ALTER TABLE
    chunk_metadata DROP COLUMN private;

ALTER TABLE
    files DROP COLUMN private;