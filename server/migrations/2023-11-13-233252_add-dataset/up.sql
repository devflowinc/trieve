-- Your SQL goes here
ALTER TABLE card_metadata
ADD COLUMN dataset TEXT NOT NULL;

UPDATE card_metadata
SET dataset = 'DEFAULT'
WHERE dataset is NULL or dataset = '';
