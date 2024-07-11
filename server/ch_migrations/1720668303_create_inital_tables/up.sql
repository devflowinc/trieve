CREATE TABLE IF NOT EXISTS dataset_events (
    id UUID,
    created_at DateTime,
    dataset_id UUID,
    event_type String,
    event_data String
) ENGINE = MergeTree()
ORDER BY (dataset_id, created_at, event_type, id)
PARTITION BY
    (toYYYYMM(created_at),
    dataset_id)
TTL created_at + INTERVAL 30 DAY;

CREATE TABLE IF NOT EXISTS search_queries
(
    id UUID,
    search_type String,
    query String,
    request_params String,
    latency Float32,
    top_score Float32,
    results Array(String),
    query_vector Array(Float32),
    dataset_id UUID,
    created_at DateTime
) ENGINE = MergeTree()
ORDER BY (dataset_id, created_at, top_score, latency, id)
PARTITION BY
    (toYYYYMM(created_at),
    dataset_id)
TTL created_at + INTERVAL 30 DAY;

CREATE TABLE IF NOT EXISTS cluster_topics
(
    id UUID,
    dataset_id UUID,
    topic String,
    density Int32,
    avg_score Float32,
    created_at DateTime
) ENGINE = MergeTree()
ORDER BY (dataset_id, id)
PARTITION BY
    dataset_id;

CREATE TABLE IF NOT EXISTS search_cluster_memberships
(
    id UUID,
    search_id UUID,
    cluster_id UUID,
    distance_to_centroid Float32,
) ENGINE = MergeTree()
ORDER BY id;

CREATE TABLE IF NOT EXISTS rag_queries (
    id UUID,
    rag_type String,
    user_message String,
    search_id UUID,
    results Array(String),
    llm_response String,
    dataset_id UUID,
    created_at DateTime,
) ENGINE = MergeTree()
ORDER BY (id, created_at)
PARTITION BY
    (toYYYYMM(created_at),
    dataset_id)
TTL created_at + INTERVAL 30 DAY;


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