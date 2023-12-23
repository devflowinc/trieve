-- Your SQL goes here
ALTER TABLE card_votes ADD COLUMN deleted boolean NOT NULL DEFAULT false;

