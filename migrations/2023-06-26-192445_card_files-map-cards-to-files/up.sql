-- Your SQL goes here
CREATE TABLE card_files (
    id uuid PRIMARY KEY,
    card_id uuid NOT NULL REFERENCES card_metadata (id),
    file_id uuid NOT NULL REFERENCES files (id),
    created_at timestamp NOT NULL DEFAULT NOW(),
    updated_at timestamp NOT NULL DEFAULT NOW()
);