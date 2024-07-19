CREATE TABLE IF NOT EXISTS last_collapsed_dataset
(
	id UUID,
	last_collapsed DateTime,
    dataset_id UUID,
    created_at DateTime
) ENGINE = MergeTree()
ORDER BY (dataset_id, created_at)
PARTITION BY
    (toYYYYMM(created_at),
    dataset_id)
TTL created_at + INTERVAL 30 DAY;

