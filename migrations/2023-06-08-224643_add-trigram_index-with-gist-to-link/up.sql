-- Your SQL goes here
CREATE INDEX idx_link_gin ON card_metadata USING gin (link gin_trgm_ops);