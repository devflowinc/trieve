-- This file should undo anything in `up.sql`
DROP TRIGGER chunk_metadata_count_trigger ON chunk_metadata;

DROP FUNCTION update_chunk_metadata_count();

DROP TABLE chunk_metadata_counts;