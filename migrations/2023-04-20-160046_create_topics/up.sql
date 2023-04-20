CREATE TABLE topics (
    id UUID NOT NULL UNIQUE PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    resolution TEXT NOT NULL,
    side BOOLEAN NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
