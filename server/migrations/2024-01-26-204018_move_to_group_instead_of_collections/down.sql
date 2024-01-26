-- This file should undo anything in `up.sql`
ALTER TABLE chunk_group RENAME TO chunk_collection;
ALTER TABLE chunk_group_bookmarks RENAME TO chunk_collection_bookmarks;
ALTER TABLE groups_from_files RENAME TO collections_from_files;
ALTER TABLE file_upload_completed_notifications RENAME COLUMN group_uuid TO collection_uuid;
ALTER TABLE collections_from_files RENAME COLUMN group_id TO collection_id;
ALTER TABLE chunk_collection_bookmarks RENAME COLUMN group_id TO collection_id;

ALTER TABLE dataset_group_counts RENAME TO dataset_collection_counts;
ALTER TABLE dataset_collection_counts RENAME COLUMN group_count TO collection_count;
