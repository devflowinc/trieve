-- Your SQL goes here
CREATE TABLE verification_notifications (
    id UUID PRIMARY KEY,
    user_uuid UUID NOT NULL REFERENCES users (id),
    chunk_uuid UUID NOT NULL REFERENCES chunk_metadata (id),
    verification_uuid UUID NOT NULL REFERENCES chunk_verification (id),
    similarity_score BigInt NOT NULL,
    user_read boolean NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
