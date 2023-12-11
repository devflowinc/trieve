-- This file should undo anything in `up.sql`
ALTER TABLE file_upload_completed_notifications DROP CONSTRAINT notifications_dataset_id_fkey;
ALTER TABLE topics DROP CONSTRAINT topics_dataset_id_fkey;
ALTER TABLE messages DROP CONSTRAINT messages_dataset_id_fkey;

ALTER TABLE file_upload_completed_notifications DROP COLUMN dataset_id;
ALTER TABLE topics DROP COLUMN dataset_id;
ALTER TABLE messages DROP COLUMN dataset_id;