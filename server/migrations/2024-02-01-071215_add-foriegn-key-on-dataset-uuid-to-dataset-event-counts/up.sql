-- Your SQL goes here
ALTER TABLE
  dataset_event_counts
ADD
  CONSTRAINT dataset_event_counts_dataset_uuid_fkey FOREIGN KEY (dataset_uuid) REFERENCES datasets (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE
  dataset_event_counts
ADD
  CONSTRAINT dataset_event_counts_dataset_uuid_unique UNIQUE (dataset_uuid);

ALTER TABLE
  dataset_event_counts
ALTER COLUMN dataset_uuid SET NOT NULL;