-- Your SQL goes here
-- Indexes for card_collection table
CREATE INDEX idx_card_collection_author_id ON card_collection (author_id);
CREATE INDEX idx_card_collection_is_public ON card_collection (is_public);
CREATE INDEX idx_card_collection_created_at ON card_collection (created_at);
CREATE INDEX idx_card_collection_updated_at ON card_collection (updated_at);

-- Indexes for card_collection_bookmarks table
CREATE INDEX idx_card_collection_bookmarks_collection_id ON card_collection_bookmarks (collection_id);
CREATE INDEX idx_card_collection_bookmarks_card_metadata_id ON card_collection_bookmarks (card_metadata_id);
CREATE INDEX idx_card_collection_bookmarks_created_at ON card_collection_bookmarks (created_at);
CREATE INDEX idx_card_collection_bookmarks_updated_at ON card_collection_bookmarks (updated_at);
