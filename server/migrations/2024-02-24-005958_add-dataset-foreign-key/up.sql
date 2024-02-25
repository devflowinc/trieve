-- Your SQL goes here

-- dataset_usage_counts
ALTER TABLE dataset_usage_counts
DROP CONSTRAINT if exists dataset_usage_counts_dataset_id_fkey;

ALTER TABLE dataset_usage_counts
ADD CONSTRAINT dataset_usage_counts_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON UPDATE CASCADE ON DELETE CASCADE;

-- events
ALTER TABLE events
DROP CONSTRAINT if exists notifications_dataset_id_fkey;

ALTER TABLE events
ADD CONSTRAINT events_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON UPDATE CASCADE ON DELETE CASCADE;

-- chunk_metadata
ALTER TABLE chunk_metadata
DROP CONSTRAINT if exists card_metadata_dataset_id_fkey;

ALTER TABLE chunk_metadata
ADD CONSTRAINT chunk_metadata_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON UPDATE CASCADE ON DELETE CASCADE;

-- chunk_group
ALTER TABLE chunk_group
DROP CONSTRAINT if exists card_collection_dataset_id_fkey;

ALTER TABLE chunk_group
ADD CONSTRAINT chunk_group_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON UPDATE CASCADE ON DELETE CASCADE;

-- files
ALTER TABLE files
DROP CONSTRAINT if exists files_dataset_id_fkey;

ALTER TABLE files
ADD CONSTRAINT files_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON UPDATE CASCADE ON DELETE CASCADE;


-- messages
ALTER TABLE messages
DROP CONSTRAINT if exists messages_dataset_id_fkey;

ALTER TABLE messages
ADD CONSTRAINT messages_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON UPDATE CASCADE ON DELETE CASCADE;

-- topics
ALTER TABLE topics
DROP CONSTRAINT if exists topics_dataset_id_fkey;

ALTER TABLE topics
ADD CONSTRAINT topics_dataset_id_fkey
FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON UPDATE CASCADE ON DELETE CASCADE;
