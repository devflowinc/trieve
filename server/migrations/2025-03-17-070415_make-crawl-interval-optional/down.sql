-- This file should undo anything in `up.sql`
ALTER TABLE crawl_requests
    ALTER COLUMN interval SET NOT NULL;

ALTER TABLE crawl_requests
    ALTER COLUMN next_crawl_at SET NOT NULL;