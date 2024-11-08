CREATE TABLE IF NOT EXISTS file_tasks (
    id String,
    pages UInt32,
    chunks UInt32,
    pages_processed UInt32,
    created_at DateTime,
    status String,
) ENGINE = MergeTree()
ORDER BY (id)
PARTITION BY
    (toYYYYMM(created_at))
TTL created_at + INTERVAL 30 DAY;

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

