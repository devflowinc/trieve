-- Your SQL goes here
ALTER TABLE crawl_requests ADD COLUMN crawl_type TEXT NOT NULL DEFAULT 'default';