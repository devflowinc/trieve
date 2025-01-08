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
    webhook_url String,
    created_at DateTime,
    updated_at DateTime
) ENGINE = MergeTree()
ORDER BY (schema_id, id)
PARTITION BY
    (schema_id);

CREATE TABLE IF NOT EXISTS inputs (
    id String,
    created_at DateTime,
    updated_at DateTime
) ENGINE = MergeTree()
ORDER BY (id);

CREATE TABLE IF NOT EXISTS batches (
    batch_id String,
    job_id String,
    output_id String,
    status String,
    created_at DateTime,
    updated_at DateTime
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY (job_id, batch_id)
PARTITION BY
    (job_id);
    