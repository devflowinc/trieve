-- This file should undo anything in `up.sql`
-- Drop the triggers
DROP TRIGGER IF EXISTS update_verification_notification_count ON verification_notifications;
DROP TRIGGER IF EXISTS update_file_upload_notification_count ON file_upload_completed_notifications;

-- Drop the function
DROP FUNCTION IF EXISTS update_notification_count();

-- Drop the table
DROP TABLE IF EXISTS user_notification_counts;
