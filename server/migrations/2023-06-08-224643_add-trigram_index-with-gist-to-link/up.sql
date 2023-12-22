-- Your SQL goes here
CREATE INDEX idx_link_gin ON chunk_metadata USING gin (link gin_trgm_ops);