-- This file should undo anything in `up.sql`
-- Remove indexes from chunk_collection table
DROP INDEX IF EXISTS idx_chunk_collection_author_id;
DROP INDEX IF EXISTS idx_chunk_collection_is_public;
DROP INDEX IF EXISTS idx_chunk_collection_created_at;
DROP INDEX IF EXISTS idx_chunk_collection_updated_at;

-- Remove indexes from chunk_collection_bookmarks table
DROP INDEX IF EXISTS idx_chunk_collection_bookmarks_collection_id;
DROP INDEX IF EXISTS idx_chunk_collection_bookmarks_chunk_metadata_id;
DROP INDEX IF EXISTS idx_chunk_collection_bookmarks_created_at;
DROP INDEX IF EXISTS idx_chunk_collection_bookmarks_updated_at;
