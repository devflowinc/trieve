-- This file should undo anything in `up.sql`
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
