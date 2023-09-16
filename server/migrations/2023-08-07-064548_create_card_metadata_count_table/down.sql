-- This file should undo anything in `up.sql`
-- Drop the trigger
DROP TRIGGER IF EXISTS card_metadata_count_trigger ON card_metadata;

-- Drop the trigger function
DROP FUNCTION IF EXISTS update_card_metadata_count();

-- Drop the card_metadata_count table
DROP TABLE IF EXISTS card_metadata_count;
