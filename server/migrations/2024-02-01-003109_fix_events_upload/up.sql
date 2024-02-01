-- Your SQL goes here
ALTER TABLE dataset_notification_counts RENAME TO dataset_event_counts;

CREATE OR REPLACE FUNCTION public.update_notification_count()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    -- Increment count when a new notification is inserted
    IF TG_OP = 'INSERT' THEN
        INSERT INTO dataset_event_counts (id, dataset_uuid, notification_count)
        VALUES (NEW.id, NEW.dataset_id, 1)
        ON CONFLICT (dataset_uuid) DO UPDATE
        SET notification_count = dataset_event_counts.notification_count + 1;
    -- Decrement count when a notification is deleted
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE dataset_event_counts
        SET notification_count = notification_count - 1
        WHERE dataset_uuid = OLD.dataset_id;
    END IF;
    RETURN NEW;
END;
$function$
;