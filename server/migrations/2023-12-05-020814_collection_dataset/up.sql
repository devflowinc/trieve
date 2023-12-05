-- Your SQL goes here
ALTER TABLE card_collection
ADD COLUMN dataset TEXT NOT NULL default 'DEFAULT';

ALTER TABLE card_collection_bookmarks
ADD COLUMN dataset TEXT NOT NULL default 'DEFAULT';
