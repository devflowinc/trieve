CREATE TABLE chunk_verification (
    id UUID PRIMARY KEY,
    chunk_id UUID NOT NULL REFERENCES chunk_metadata(id),
    similarity_score BigInt NOT NULL
);
