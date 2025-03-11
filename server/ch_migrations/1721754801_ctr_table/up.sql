CREATE TABLE IF NOT EXISTS ctr_data on CLUSTER `{cluster}`
(
    id UUID,
    request_id UUID,
    chunk_id UUID,
    dataset_id UUID,
    position Int32,
    metadata String,
    created_at DateTime,
) ENGINE = ReplicatedMergeTree()
ORDER BY (dataset_id, created_at, request_id, id)
PARTITION BY
    (toYYYYMM(created_at),
    dataset_id)
TTL created_at + INTERVAL 30 DAY;
