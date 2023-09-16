-- Remove the index
DROP INDEX IF EXISTS idx_card_metadata_tsvector;

-- Remove the column
ALTER TABLE card_metadata
DROP COLUMN IF EXISTS card_metadata_tsvector;