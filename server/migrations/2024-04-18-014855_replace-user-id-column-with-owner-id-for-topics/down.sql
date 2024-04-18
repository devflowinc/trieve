-- This file should undo anything in `up.sql`
ALTER TABLE topics ADD COLUMN user_id TEXT;

UPDATE topics
SET user_id = owner_id;

ALTER TABLE topics DROP COLUMN owner_id;

DROP INDEX IF EXISTS "topics_dataset_id_owner_id_idx";
