-- This file should undo anything in `up.sql`
-- Finally, let's drop the trigger and function
DROP TRIGGER IF EXISTS trigger_update_dataset_timestamp ON chunk_metadata;
DROP FUNCTION IF EXISTS update_dataset_timestamp();