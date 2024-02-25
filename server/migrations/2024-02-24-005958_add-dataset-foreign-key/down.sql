-- This file should undo anything in `up.sql`

-- dataset_usage_counts
ALTER TABLE dataset_usage_counts
DROP CONSTRAINT if exists datasets_usage_counts_dataset_id_fkey;

ALTER TABLE dataset_usage_counts
ADD CONSTRAINT datasets_usage_counts_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id);

-- events
ALTER TABLE events
DROP CONSTRAINT if exists events_dataset_id_fkey;

ALTER TABLE events
ADD CONSTRAINT notifications_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id);

-- chunk_metadata
ALTER TABLE chunk_metadata
DROP CONSTRAINT if exists chunk_metadata_dataset_id_fkey;

ALTER TABLE chunk_metadata
ADD CONSTRAINT card_metadata_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id);

-- chunk_group
ALTER TABLE chunk_group
DROP CONSTRAINT if exists chunk_group_dataset_id_fkey;

ALTER TABLE chunk_group
ADD CONSTRAINT card_collection_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id);

-- files
ALTER TABLE files
DROP CONSTRAINT if exists files_dataset_id_fkey;

ALTER TABLE files
ADD CONSTRAINT files_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id);


-- messages
ALTER TABLE messages
DROP CONSTRAINT if exists messages_dataset_id_fkey;

ALTER TABLE messages
ADD CONSTRAINT messages_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id);

-- topics
ALTER TABLE topics
DROP CONSTRAINT if exists topics_dataset_id_fkey;

ALTER TABLE topics
ADD CONSTRAINT topics_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id);
