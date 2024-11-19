DROP TABLE  IF EXISTS file_chunks;
CREATE TABLE IF NOT EXISTS file_chunks (
    id String,
    task_id String,
    content String,
    metadata String,
    created_at DateTime,
) ENGINE = MergeTree()
ORDER BY (task_id, id)
PARTITION BY
    (task_id)
TTL created_at + INTERVAL 30 DAY;