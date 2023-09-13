-- Your SQL goes here
CREATE OR REPLACE FUNCTION update_notification_count()
RETURNS TRIGGER AS $$
BEGIN
    -- Increment count when a new notification is inserted
    IF TG_OP = 'INSERT' THEN
        INSERT INTO user_notification_counts (id, user_id, notification_count)
        VALUES (NEW.id, NEW.author_id, 1)
        ON CONFLICT (user_id) DO UPDATE
        SET notification_count = user_notification_counts.notification_count + 1;
    -- Decrement count when a notification is deleted
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE user_notification_counts
        SET notification_count = notification_count - 1
        WHERE user_uuid = OLD.user_uuid;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
