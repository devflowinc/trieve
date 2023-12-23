-- This file should undo anything in `up.sql`
ALTER TABLE chunk_collection
RENAME TO card_collection;

ALTER TABLE chunk_collection_bookmarks
RENAME COLUMN chunk_collection_id TO card_collection_id
RENAME TO card_collection_bookmarks;

ALTER TABLE chunk_collisions
RENAME COLUMN chunk_id TO card_id
RENAME TO card_collisions;

ALTER TABLE chunk_files
RENAME COLUMN chunk_id TO card_id
RENAME TO card_files;

ALTER TABLE chunk_metadata
RENAME COLUMN chunk_html TO card_html
RENAME TO card_metadata;

ALTER TABLE chunk_metadata_counts
RENAME TO card_metadata_counts;

ALTER TABLE cut_chunks
RENAME COLUMN cut_chunk_content TO cut_card_content;
RENAME TO cut_cards;

ALTER TABLE stripe_plans
RENAME COLUMN chunk_count TO card_count;
