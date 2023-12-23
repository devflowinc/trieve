-- This file should undo anything in `up.sql`
ALTER TABLE stripe_plans
RENAME COLUMN chunk_count TO card_count;

ALTER TABLE cut_chunks
RENAME TO cut_cards;

ALTER TABLE cut_cards
RENAME COLUMN cut_chunk_content TO cut_card_content;

ALTER TABLE chunk_metadata
RENAME TO card_metadata;

ALTER TABLE card_metadata
RENAME COLUMN chunk_html TO card_html;

ALTER TABLE chunk_metadata_counts
RENAME TO card_metadata_counts;

ALTER TABLE chunk_files
RENAME TO card_files;

ALTER TABLE card_files
RENAME COLUMN chunk_id TO card_id;

ALTER TABLE chunk_collisions
RENAME TO card_collisions;

ALTER TABLE card_collisions
RENAME COLUMN chunk_id TO card_id;

ALTER TABLE chunk_collection_bookmarks
RENAME TO card_collection_bookmarks;

ALTER TABLE card_collection_bookmarks
RENAME COLUMN chunk_metadata_id TO card_metadata_id;

ALTER TABLE chunk_collection
RENAME TO card_collection;
