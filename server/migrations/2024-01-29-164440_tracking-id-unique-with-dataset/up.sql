-- Drop the existing unique constraint on tracking_id
ALTER TABLE chunk_metadata
DROP CONSTRAINT card_metadata_tracking_id_key;

-- Add a new unique constraint on tracking_id and dataset_id
ALTER TABLE chunk_metadata
ADD CONSTRAINT chunk_metadata_tracking_id_dataset_id_key UNIQUE (tracking_id, dataset_id);
