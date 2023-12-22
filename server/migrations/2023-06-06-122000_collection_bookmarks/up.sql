-- Your SQL goes here

CREATE TABLE chunk_collection_bookmarks (
    id UUID PRIMARY KEY,
    collection_id UUID NOT NULL REFERENCES chunk_collection(id),
    chunk_metadata_id UUID NOT NULL REFERENCES chunk_metadata(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
