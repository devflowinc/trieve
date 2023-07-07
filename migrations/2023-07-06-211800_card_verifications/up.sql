CREATE TABLE card_verification (
    id UUID PRIMARY KEY,
    card_id UUID NOT NULL REFERENCES card_metadata(id),
    similarity_score BigInt NOT NULL
);
