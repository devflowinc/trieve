CREATE TABLE IF NOT EXISTS words_datasets on CLUSTER `{cluster}`
(
    id UUID NOT NULL,
    dataset_id UUID NOT NULL,
    word String NOT NULL,
    count Int32 NOT NULL,
    created_at DateTime DEFAULT now() NOT NULL,
    INDEX idx_created_at created_at TYPE minmax GRANULARITY 8192,
    INDEX idx_id id TYPE minmax GRANULARITY 8192
) ENGINE = ReplicatedSummingMergeTree(created_at)
ORDER BY (dataset_id, word)
PARTITION BY dataset_id;

