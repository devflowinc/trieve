-- This file should undo anything in `up.sql`
ALTER TABLE
    card_metadata DROP COLUMN IF EXISTS dataset_id;

ALTER TABLE
    card_collection DROP COLUMN IF EXISTS dataset_id;