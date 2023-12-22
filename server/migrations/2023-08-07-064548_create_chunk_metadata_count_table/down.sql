-- This file should undo anything in `up.sql`
-- Drop the trigger
DROP TRIGGER IF EXISTS chunk_metadata_count_trigger ON chunk_metadata;

-- Drop the trigger function
DROP FUNCTION IF EXISTS update_chunk_metadata_count();

-- Drop the chunk_metadata_count table
DROP TABLE IF EXISTS chunk_metadata_count;
