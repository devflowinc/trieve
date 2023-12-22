-- Your SQL goes here
CREATE INDEX json_gin ON chunk_metadata USING gin (metadata);
