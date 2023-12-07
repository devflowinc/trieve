-- Your SQL goes here
CREATE TABLE organizations (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT UNIQUE NOT NULL,
    configuration JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO organizations (name, configuration) VALUES ('DEFAULT', '{}');

CREATE INDEX idx_organizations_name ON organizations (name);