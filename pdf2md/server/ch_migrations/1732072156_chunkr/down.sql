ALTER TABLE file_tasks
DROP COLUMN IF EXISTS provider;

ALTER TABLE file_tasks
DROP COLUMN IF EXISTS chunkr_task_id;
