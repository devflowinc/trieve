-- This file should undo anything in `up.sql`
ALTER TABLE events DROP COLUMN event_type;
ALTER TABLE events DROP COLUMN event_data;
ALTER TABLE events ADD COLUMN user_read BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE events ADD COLUMN collection_id UUID;
ALTER TABLE events RENAME TO file_upload_completed_notifications;