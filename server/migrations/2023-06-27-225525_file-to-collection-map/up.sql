-- Your SQL goes here
CREATE TABLE collections_from_files (
    id UUID PRIMARY KEY,
    collection_id UUID NOT NULL REFERENCES chunk_collection (id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files (id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);