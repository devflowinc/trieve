-- Your SQL goes here
CREATE TABLE organizations (
    id UUID PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    configuration JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_organizations_name ON organizations (name);