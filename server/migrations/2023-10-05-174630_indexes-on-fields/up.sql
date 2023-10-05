-- Your SQL goes here
CREATE INDEX json_gin ON card_metadata USING gin (metadata);
