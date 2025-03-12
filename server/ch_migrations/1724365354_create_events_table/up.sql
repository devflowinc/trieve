CREATE TABLE IF NOT EXISTS events on CLUSTER `{cluster}`
(
    id UUID,
    event_type String,
    event_name String,
    items Array(String),
    metadata String,
    user_id String,
    is_conversion Bool,
    request_id String,
    dataset_id UUID,
    created_at DateTime DEFAULT now(),
    updated_at DateTime DEFAULT now(),
)
ENGINE = ReplicatedMergeTree()
ORDER BY (event_type, created_at, id)
PARTITION BY
    (dataset_id);
