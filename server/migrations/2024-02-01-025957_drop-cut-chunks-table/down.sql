-- This file should undo anything in `up.sql`
CREATE TABLE cut_chunks (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id),
    cut_card_content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
)
