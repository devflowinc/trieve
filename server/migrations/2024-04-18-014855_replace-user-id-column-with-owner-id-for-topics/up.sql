-- Your SQL goes here
ALTER TABLE topics
ADD COLUMN owner_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

UPDATE topics
SET owner_id = user_id
WHERE user_id IS NOT NULL;

ALTER TABLE topics
DROP COLUMN user_id;

CREATE INDEX topics_dataset_id_owner_id_idx
ON topics (dataset_id, owner_id);

