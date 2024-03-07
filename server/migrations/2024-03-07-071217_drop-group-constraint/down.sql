-- This file should undo anything in `up.sql`
ALTER TABLE chunk_group
ADD CONSTRAINT chunk_group_tracking_id_key UNIQUE (tracking_id);
