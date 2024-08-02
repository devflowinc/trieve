-- Your SQL goes here
CREATE INDEX chunk_metadata_created_at_idx ON chunk_metadata (created_at);
CREATE INDEX groups_created_at_idx ON chunk_group (created_at);
CREATE INDEX files_created_at_idx ON files (created_at);