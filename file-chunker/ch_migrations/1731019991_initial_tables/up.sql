CREATE TABLE IF NOT EXISTS file_tasks (
    id String,
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
    text String,
    metadata String,
    created_at DateTime,
) ENGINE = MergeTree()