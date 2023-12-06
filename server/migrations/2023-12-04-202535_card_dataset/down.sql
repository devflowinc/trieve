DROP TABLE IF EXISTS dataset;

-- This file should undo anything in `up.sql`
ALTER TABLE card_metadata
DROP COLUMN IF EXISTS dataset_id;

-- This file should undo anything in `up.sql`
ALTER TABLE card_collection
DROP COLUMN IF EXISTS dataset_id;

-- This file should undo anything in `up.sql`
ALTER TABLE card_collection_bookmarks
DROP COLUMN IF EXISTS dataset_id;
