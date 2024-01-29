-- This file should undo anything in `up.sql`

-- Drop the new unique constraint on tracking_id and dataset_id
ALTER TABLE chunk_metadata
DROP CONSTRAINT chunk_metadata_tracking_id_dataset_id_key;

-- Recreate the dropped unique constraint on tracking_id
ALTER TABLE chunk_metadata
ADD CONSTRAINT card_metadata_tracking_id_key UNIQUE (tracking_id);

