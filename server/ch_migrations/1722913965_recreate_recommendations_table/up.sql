CREATE TABLE IF NOT EXISTS recommendations (
    id UUID,
    recommendation_type String,
    positive_ids Array(String),
    negative_ids Array(String),
    positive_tracking_ids Array(String),
    negative_tracking_ids Array(String),
    request_params String,
    results Array(String),
    top_score Float32,
    dataset_id UUID,
    created_at DateTime,
) ENGINE = MergeTree()
ORDER BY (id, created_at)
PARTITION BY
    (toYYYYMM(created_at),
    dataset_id)
TTL created_at + INTERVAL 30 DAY;
