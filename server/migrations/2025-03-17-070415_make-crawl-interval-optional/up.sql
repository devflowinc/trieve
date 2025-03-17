-- Your SQL goes here
ALTER TABLE crawl_requests
    ALTER COLUMN interval DROP NOT NULL;

ALTER TABLE crawl_requests
    ALTER COLUMN next_crawl_at DROP NOT NULL;
