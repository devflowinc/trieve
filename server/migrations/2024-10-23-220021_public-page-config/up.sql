-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public_page_configuration (
    id UUID PRIMARY KEY,
    dataset_id UUID NOT NULL UNIQUE REFERENCES datasets(id) ON DELETE CASCADE,
    is_public boolean NOT NULL default false,
    api_key Text NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

