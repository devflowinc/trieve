-- Your SQL goes here
CREATE TABLE IF NOT EXISTS crawl_requests (
    id UUID PRIMARY KEY,
    url TEXT NOT NULL,
    status TEXT NOT NULL,
    interval INT NOT NULL,
    next_crawl_at TIMESTAMP NOT NULL,
    scrape_id UUID NOT NULL,
    dataset_id UUID NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL
);

