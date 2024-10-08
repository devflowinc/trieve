CREATE TABLE IF NOT EXISTS events (
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
ORDER BY (event_type, created_at, id)
PARTITION BY
    (toYYYYMM(created_at),
    dataset_id);
