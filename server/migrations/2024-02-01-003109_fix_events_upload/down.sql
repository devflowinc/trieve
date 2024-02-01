-- This file should undo anything in `up.sql`
;-- Your SQL goes here
DROP FUNCTION IF EXISTS public.update_notification_count() CASCADE;

-- Revert table name change
ALTER TABLE dataset_event_counts RENAME TO dataset_notification_counts;