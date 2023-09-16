-- This file should undo anything in `up.sql`
-- Drop the trigger
DROP TRIGGER IF EXISTS update_tsvector_trigger ON card_metadata;

-- Drop the trigger function
DROP FUNCTION IF EXISTS update_tsvector();