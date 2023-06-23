-- Your SQL goes here
CREATE TABLE files (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id),
    file_name TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    private BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
