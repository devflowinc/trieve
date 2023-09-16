-- Your SQL goes here

CREATE TABLE card_collection_bookmarks (
    id UUID PRIMARY KEY,
    collection_id UUID NOT NULL REFERENCES card_collection(id),
    card_metadata_id UUID NOT NULL REFERENCES card_metadata(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
