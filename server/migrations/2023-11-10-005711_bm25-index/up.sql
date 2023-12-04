-- Your SQL goes here
-- CREATE EXTENSION IF NOT EXISTS pg_bm25;

-- CREATE INDEX idx_content_search
-- ON card_metadata
-- USING bm25 ((card_metadata.*))
-- WITH (text_fields='{"card_html": {}}');