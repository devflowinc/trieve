-- Your SQL goes here
ALTER TABLE datasets
ADD COLUMN configuration JSONB NOT NULL DEFAULT '{}'::JSONB;
