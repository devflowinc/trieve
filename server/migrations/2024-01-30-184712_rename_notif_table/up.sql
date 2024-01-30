-- Your SQL goes here
ALTER TABLE file_upload_completed_notifications RENAME TO events;
ALTER TABLE events DROP COLUMN user_read;
ALTER TABLE events DROP COLUMN group_uuid;
ALTER TABLE events ADD COLUMN event_type VARCHAR(255) NOT NULL;
ALTER TABLE events ADD COLUMN event_data JSONB NOT NULL;
