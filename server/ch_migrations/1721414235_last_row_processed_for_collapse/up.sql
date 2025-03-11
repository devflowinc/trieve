CREATE TABLE IF NOT EXISTS last_collapsed_dataset on CLUSTER `{cluster}`
(
    id UUID,
    last_collapsed DateTime,
    dataset_id UUID,
    created_at DateTime
)
ENGINE = ReplicatedReplacingMergeTree(created_at)
ORDER BY (dataset_id)
PARTITION BY (toYYYYMM(created_at), dataset_id);

