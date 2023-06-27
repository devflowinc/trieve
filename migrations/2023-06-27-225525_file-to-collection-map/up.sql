-- Your SQL goes here
CREATE TABLE collections_from_files (
    id UUID PRIMARY KEY,
    collection_id UUID NOT NULL REFERENCES card_collection (id),
    file_id UUID NOT NULL REFERENCES card_files (id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);