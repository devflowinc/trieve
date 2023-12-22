-- This file should undo anything in `up.sql`
-- Drop the trigger
DROP TRIGGER IF EXISTS update_collection_counts_trigger ON chunk_collection;

-- Drop the function
DROP FUNCTION IF EXISTS update_collection_counts();

-- Drop the user_collection_count table
DROP TABLE IF EXISTS user_collection_counts;
