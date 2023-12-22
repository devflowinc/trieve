-- Your SQL goes here
ALTER TABLE chunk_metadata
ADD COLUMN chunk_metadata_tsvector TSVECTOR;

UPDATE chunk_metadata
SET chunk_metadata_tsvector = to_tsvector('english', content);

CREATE INDEX idx_chunk_metadata_tsvector ON chunk_metadata USING GIN(chunk_metadata_tsvector);
