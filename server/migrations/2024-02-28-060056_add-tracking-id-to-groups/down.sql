-- This file should undo anything in `up.sql`
ALTER TABLE chunk_group
DROP COLUMN tracking_id;
ALTER TABLE chunk_group
DROP CONSTRAINT chunk_group_tracking_id_dataset_id_key;