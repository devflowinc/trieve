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


ALTER TABLE chunk_metadata ADD COLUMN IF NOT EXISTS chunk_metadata_tsvector tsvector;

CREATE FUNCTION IF NOT EXISTS update_tsvector() RETURNS TRIGGER AS $$
BEGIN
    NEW.chunk_metadata_tsvector := to_tsvector(NEW.content);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER IF NOT EXISTS update_tsvector_trigger
BEFORE INSERT ON chunk_metadata
FOR EACH ROW
EXECUTE FUNCTION update_tsvector();
