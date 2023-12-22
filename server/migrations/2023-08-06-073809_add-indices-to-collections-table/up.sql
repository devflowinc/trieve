-- Your SQL goes here
-- Indexes for chunk_collection table
CREATE INDEX idx_chunk_collection_author_id ON chunk_collection (author_id);
CREATE INDEX idx_chunk_collection_is_public ON chunk_collection (is_public);
CREATE INDEX idx_chunk_collection_created_at ON chunk_collection (created_at);
CREATE INDEX idx_chunk_collection_updated_at ON chunk_collection (updated_at);

-- Indexes for chunk_collection_bookmarks table
CREATE INDEX idx_chunk_collection_bookmarks_collection_id ON chunk_collection_bookmarks (collection_id);
CREATE INDEX idx_chunk_collection_bookmarks_chunk_metadata_id ON chunk_collection_bookmarks (chunk_metadata_id);
CREATE INDEX idx_chunk_collection_bookmarks_created_at ON chunk_collection_bookmarks (created_at);
CREATE INDEX idx_chunk_collection_bookmarks_updated_at ON chunk_collection_bookmarks (updated_at);
