-- Your SQL goes here

CREATE TABLE card_collection (
    id UUID PRIMARY KEY,
    author_id UUID NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    description TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
