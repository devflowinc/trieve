-- Your SQL goes here
CREATE INDEX idx_content_gist ON card_metadata USING gist (content gist_trgm_ops);
