-- Your SQL goes here
ALTER TABLE crawl_requests
ADD COLUMN crawl_options JSONB NOT NULL DEFAULT '{}';