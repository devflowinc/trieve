-- This file should undo anything in `up.sql`
CREATE TABLE chunk_votes (
    id UUID PRIMARY KEY,
    voted_user_id UUID NOT NULL REFERENCES users(id),
    chunk_metadata_id UUID NOT NULL REFERENCES chunk_metadata(id),
    vote BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chunk_votes_voted_user_id ON chunk_votes (voted_user_id);
CREATE INDEX idx_chunk_votes_chunk_metadata_id ON chunk_votes (chunk_metadata_id);
