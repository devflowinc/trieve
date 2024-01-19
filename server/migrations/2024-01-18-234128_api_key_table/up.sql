-- Your SQL goes here
CREATE TABLE user_api_key (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    api_key_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL DEFAULT 'default',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE

);

ALTER TABLE users DROP COLUMN api_key_hash;