-- Your SQL goes here
ALTER TABLE file_upload_completed_notifications ADD COLUMN dataset_id UUID NOT NULL;
ALTER TABLE topics ADD COLUMN dataset_id UUID NOT NULL;
ALTER TABLE messages ADD COLUMN dataset_id UUID NOT NULL;

ALTER TABLE file_upload_completed_notifications ADD CONSTRAINT notifications_dataset_id_fkey FOREIGN KEY (dataset_id) REFERENCES datasets(id);
ALTER TABLE topics ADD CONSTRAINT topics_dataset_id_fkey FOREIGN KEY (dataset_id) REFERENCES datasets(id);
ALTER TABLE messages ADD CONSTRAINT messages_dataset_id_fkey FOREIGN KEY (dataset_id) REFERENCES datasets(id);
