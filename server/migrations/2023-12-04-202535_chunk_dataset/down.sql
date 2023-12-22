DROP TABLE IF EXISTS datasets;

-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata
DROP COLUMN IF EXISTS dataset_id;

-- This file should undo anything in `up.sql`
ALTER TABLE chunk_collection
DROP COLUMN IF EXISTS dataset_id;

-- This file should undo anything in `up.sql`
ALTER TABLE chunk_collection_bookmarks
DROP COLUMN IF EXISTS dataset_id;
