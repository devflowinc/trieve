-- Your SQL goes here
ALTER TABLE chunk_votes ADD COLUMN deleted boolean NOT NULL DEFAULT false;

