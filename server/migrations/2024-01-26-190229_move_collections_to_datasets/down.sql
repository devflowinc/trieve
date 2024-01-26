-- This file should undo anything in `up.sql`
ALTER TABLE chunk_collections ADD COLUMN author_id uuid;
ALTER TABLE user_collection_counts ADD COLUMN user_id uuid;
ALTER TABLE chunk_collections DROP COLUMN dataset_id;
ALTER TABLE dataset_collection_counts RENAME TO user_collection_counts;