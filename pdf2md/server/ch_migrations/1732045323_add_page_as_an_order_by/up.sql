DROP TABLE IF EXISTS default.file_chunks;
CREATE TABLE IF NOT EXISTS file_chunks (
    id String,
    task_id String,
    content String,
    usage String,
    page UInt32,
    created_at DateTime,
) ENGINE = MergeTree()
ORDER BY (task_id, page, id)
PARTITION BY
    (task_id)
TTL created_at + INTERVAL 30 DAY;