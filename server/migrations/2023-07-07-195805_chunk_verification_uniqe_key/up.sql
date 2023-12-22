-- Your SQL goes here
ALTER TABLE chunk_verification ADD CONSTRAINT uq_chunk_id UNIQUE(chunk_id)
