-- up.sql
ALTER TABLE card_collection
RENAME TO chunk_collection;

ALTER TABLE card_collection_bookmarks
RENAME COLUMN card_metadata_id TO chunk_metadata_id;

ALTER TABLE card_collection_bookmarks
RENAME TO chunk_collection_bookmarks;

ALTER TABLE card_collisions
RENAME COLUMN card_id TO chunk_id;

ALTER TABLE card_collisions
RENAME TO chunk_collisions;

ALTER TABLE card_files
RENAME COLUMN card_id TO chunk_id;

ALTER TABLE card_files
RENAME TO chunk_files;

ALTER TABLE card_metadata
RENAME COLUMN card_html TO chunk_html;

ALTER TABLE card_metadata
RENAME TO chunk_metadata;

ALTER TABLE card_metadata_counts
RENAME TO chunk_metadata_counts;

ALTER TABLE cut_cards
RENAME COLUMN cut_card_content TO cut_chunk_content;

ALTER TABLE cut_cards
RENAME TO cut_chunks;

ALTER TABLE stripe_plans
RENAME COLUMN card_count TO chunk_count;
