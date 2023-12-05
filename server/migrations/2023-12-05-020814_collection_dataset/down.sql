-- This file should undo anything in `up.sql`
ALTER TABLE card_collection
DROP COLUMN IF EXISTS dataset;
