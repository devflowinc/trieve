-- This file should undo anything in `up.sql`

-- Delete indexes for file_upload_completed_notifications table
DROP INDEX idx_file_upload_completed_notifications_user_uuid;
DROP INDEX idx_file_upload_completed_notifications_collection_uuid;

-- Delete indexes for verification_notifications table
DROP INDEX idx_verification_notifications_user_uuid;
DROP INDEX idx_verification_notifications_card_uuid;
DROP INDEX idx_verification_notifications_verification_uuid;
