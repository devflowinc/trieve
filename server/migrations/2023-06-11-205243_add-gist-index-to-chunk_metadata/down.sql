-- Remove the index
DROP INDEX IF EXISTS idx_chunk_metadata_tsvector;

-- Remove the column
ALTER TABLE chunk_metadata
DROP COLUMN IF EXISTS chunk_metadata_tsvector;