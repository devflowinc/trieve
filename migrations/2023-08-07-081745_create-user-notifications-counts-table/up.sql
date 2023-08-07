-- Your SQL goes here
CREATE TABLE user_notification_counts (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    user_uuid UUID UNIQUE NOT NULL REFERENCES users(id),
    notification_count INTEGER NOT NULL DEFAULT 0
);

CREATE OR REPLACE FUNCTION update_notification_count()
RETURNS TRIGGER AS $$
BEGIN
    -- Increment count when a new notification is inserted
    IF TG_OP = 'INSERT' THEN
        UPDATE user_notification_counts
        SET notification_count = notification_count + 1
        WHERE user_uuid = NEW.user_uuid;
    -- Decrement count when a notification is deleted
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE user_notification_counts
        SET notification_count = notification_count - 1
        WHERE user_uuid = OLD.user_uuid;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for verification_notifications
CREATE TRIGGER update_verification_notification_count
AFTER INSERT OR DELETE ON verification_notifications
FOR EACH ROW
EXECUTE FUNCTION update_notification_count();

-- Trigger for file_upload_completed_notifications
CREATE TRIGGER update_file_upload_notification_count
AFTER INSERT OR DELETE ON file_upload_completed_notifications
FOR EACH ROW
EXECUTE FUNCTION update_notification_count();

-- Initialize user_notification_counts with existing data
INSERT INTO user_notification_counts (id, user_uuid, notification_count)
SELECT DISTINCT ON (user_uuid) gen_random_uuid(), user_uuid, 
    ((SELECT COUNT(*) FROM verification_notifications WHERE user_uuid = user_uuid) + 
        (SELECT COUNT(*) FROM file_upload_completed_notifications WHERE user_uuid = user_uuid))
FROM file_upload_completed_notifications;