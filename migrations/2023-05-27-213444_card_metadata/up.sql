-- Your SQL goes here
CREATE TABLE card_metadata (
    id UUID PRIMARY KEY,
    content TEXT NOT NULL,
    author_id UUID NOT NULL REFERENCES users(id),
    qdrant_point_id UUID NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE card_upvotes (
    id UUID PRIMARY KEY,
    voted_user_id UUID NOT NULL REFERENCES users(id),
    card_metadata_id UUID NOT NULL REFERENCES card_metadata(id),
    vote BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
