-- Your SQL goes here
CREATE TABLE card_metadata (
    id UUID PRIMARY KEY,
    content TEXT NOT NULL,
    link TEXT DEFAULT NULL,
    author_id UUID NOT NULL REFERENCES users(id),
    qdrant_point_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_card_metadata_author_id ON card_metadata (author_id);
CREATE INDEX idx_card_metadata_qdrant_point_id ON card_metadata (qdrant_point_id);

CREATE TABLE card_votes (
    id UUID PRIMARY KEY,
    voted_user_id UUID NOT NULL REFERENCES users(id),
    card_metadata_id UUID NOT NULL REFERENCES card_metadata(id),
    vote BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_card_votes_voted_user_id ON card_votes (voted_user_id);
CREATE INDEX idx_card_votes_card_metadata_id ON card_votes (card_metadata_id);
