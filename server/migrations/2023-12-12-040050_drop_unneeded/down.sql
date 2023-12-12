-- This file should undo anything in `up.sql`
CREATE TABLE invitations (
  id UUID NOT NULL UNIQUE PRIMARY KEY,
  email VARCHAR(100) NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

CREATE TABLE password_resets (
    id UUID NOT NULL UNIQUE PRIMARY KEY,
    email VARCHAR(100) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);


ALTER TABLE card_metadata ADD COLUMN IF NOT EXISTS card_metadata_tsvector tsvector;
