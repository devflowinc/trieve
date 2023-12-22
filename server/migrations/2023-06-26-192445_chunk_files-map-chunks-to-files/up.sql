-- Your SQL goes here
CREATE TABLE chunk_files (
    id uuid PRIMARY KEY,
    chunk_id uuid NOT NULL REFERENCES chunk_metadata (id),
    file_id uuid NOT NULL REFERENCES files (id),
    created_at timestamp NOT NULL DEFAULT NOW(),
    updated_at timestamp NOT NULL DEFAULT NOW()
);
