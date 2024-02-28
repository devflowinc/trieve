-- Your SQL goes here
ALTER TABLE chunk_group
ADD COLUMN tracking_id TEXT UNIQUE;

ALTER TABLE chunk_group
ADD CONSTRAINT chunk_group_tracking_id_dataset_id_key UNIQUE (tracking_id, dataset_id);