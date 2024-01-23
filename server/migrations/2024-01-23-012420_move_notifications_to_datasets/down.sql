-- This file should undo anything in `up.sql`
-- Your SQL goes here
ALTER TABLE dataset_notification_counts ADD COLUMN user_uuid uuid;
ALTER TABLE dataset_notification_counts DROP COLUMN dataset_uuid;
ALTER TABLE dataset_notification_counts RENAME TO user_notification_counts;
ALTER TABLE file_upload_completed_notifications ADD COLUMN user_uuid uuid;