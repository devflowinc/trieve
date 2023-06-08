-- Your SQL goes here
CREATE EXTENSION pg_trgm;
CREATE INDEX idx_gist ON card_metadata USING gist (oc_file_path gist_trgm_ops);
