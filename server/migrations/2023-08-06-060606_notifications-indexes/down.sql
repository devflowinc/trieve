-- This file should undo anything in `up.sql`

-- Delete indexes for file_upload_completed_notifications table
DROP INDEX idx_file_upload_completed_notifications_user_uuid;
DROP INDEX idx_file_upload_completed_notifications_collection_uuid;


