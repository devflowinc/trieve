CREATE TABLE IF NOT EXISTS schemas (
    id String,
    name String,
    schema String,
    created_at DateTime,
    updated_at DateTime
) ENGINE = MergeTree()
ORDER BY (id)
PARTITION BY
    (toYYYYMM(created_at));

CREATE TABLE IF NOT EXISTS jobs (
    id String,
    schema_id String,
    input_id String,
    status String,
    batch_id String,
    output_id String,
    created_at DateTime,
    updated_at DateTime
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY (schema_id, id)
PARTITION BY
    (schema_id);

CREATE TABLE IF NOT EXISTS inputs (
    id String,
    created_at DateTime,
    updated_at DateTime
) ENGINE = MergeTree()
ORDER BY (id);
