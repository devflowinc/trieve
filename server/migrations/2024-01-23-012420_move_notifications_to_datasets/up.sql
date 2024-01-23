-- Your SQL goes here
ALTER TABLE file_upload_completed_notifications DROP COLUMN user_uuid;

-- Assuming you want to rename the table, use RENAME TO
ALTER TABLE user_notification_counts RENAME TO dataset_notification_counts;

-- Assuming you want to add a new column 'dataset_uuid' with type 'uuid'
ALTER TABLE dataset_notification_counts ADD COLUMN dataset_uuid uuid;

-- Assuming you want to drop the column 'user_uuid' from 'dataset_notification_count'
ALTER TABLE dataset_notification_counts DROP COLUMN user_uuid;

CREATE OR REPLACE FUNCTION update_notification_count()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    -- Increment count when a new notification is inserted
    IF TG_OP = 'INSERT' THEN
        INSERT INTO dataset_notification_counts (id, dataset_uuid, notification_count)
        VALUES (NEW.id, NEW.dataset_uuid, 1)
        ON CONFLICT (dataset_uuid) DO UPDATE
        SET notification_count = dataset_notification_counts.notification_count + 1;
    -- Decrement count when a notification is deleted
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE dataset_notification_counts
        SET notification_count = notification_count - 1
        WHERE user_uuid = OLD.dataset_uuid;
    END IF;
    RETURN NEW;
END;
$function$
;