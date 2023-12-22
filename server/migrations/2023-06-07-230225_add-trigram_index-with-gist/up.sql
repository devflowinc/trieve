-- Your SQL goes here
CREATE EXTENSION pg_trgm;
CREATE INDEX idx_gist ON chunk_metadata USING gin (oc_file_path gin_trgm_ops);
