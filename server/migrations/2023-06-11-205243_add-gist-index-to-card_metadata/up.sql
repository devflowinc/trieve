-- Your SQL goes here
ALTER TABLE card_metadata
ADD COLUMN card_metadata_tsvector TSVECTOR;

UPDATE card_metadata
SET card_metadata_tsvector = to_tsvector('english', content);

CREATE INDEX idx_card_metadata_tsvector ON card_metadata USING GIN(card_metadata_tsvector);
