-- This file should undo anything in `up.sql`
ALTER TABLE dataset_event_counts DROP CONSTRAINT dataset_event_counts_dataset_uuid_fkey;

ALTER TABLE dataset_event_counts DROP CONSTRAINT dataset_event_counts_dataset_uuid_unique;
