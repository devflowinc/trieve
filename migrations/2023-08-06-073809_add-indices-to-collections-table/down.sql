-- This file should undo anything in `up.sql`
-- Remove indexes from card_collection table
DROP INDEX IF EXISTS idx_card_collection_author_id;
DROP INDEX IF EXISTS idx_card_collection_is_public;
DROP INDEX IF EXISTS idx_card_collection_created_at;
DROP INDEX IF EXISTS idx_card_collection_updated_at;

-- Remove indexes from card_collection_bookmarks table
DROP INDEX IF EXISTS idx_card_collection_bookmarks_collection_id;
DROP INDEX IF EXISTS idx_card_collection_bookmarks_card_metadata_id;
DROP INDEX IF EXISTS idx_card_collection_bookmarks_created_at;
DROP INDEX IF EXISTS idx_card_collection_bookmarks_updated_at;
