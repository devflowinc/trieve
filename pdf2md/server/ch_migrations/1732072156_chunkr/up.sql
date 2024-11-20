ALTER TABLE file_tasks
ADD COLUMN IF NOT EXISTS provider String;

ALTER TABLE file_tasks
ADD COLUMN IF NOT EXISTS chunkr_task_id String;

ALTER TABLE file_tasks
ADD COLUMN IF NOT EXISTS chunkr_api_key Nullable(String);
