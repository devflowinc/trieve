-- This file should undo anything in `up.sql`
DROP INDEX idx_chunk_metadata_json;
CREATE INDEX idx_card_metadata_link ON chunk_metadata (link);
CREATE INDEX idx_chunk_metadata_created_at ON chunk_metadata (created_at);
CREATE INDEX idx_chunk_metadata_time_stamp ON chunk_metadata (time_stamp);