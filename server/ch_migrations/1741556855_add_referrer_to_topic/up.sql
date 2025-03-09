CREATE TABLE IF NOT EXISTS default.topics
(

    `id` UUID,

    `name` String,

    `topic_id` UUID,

    `dataset_id` UUID,

    `owner_id` String,

    `referrer` String,

    `created_at` DateTime DEFAULT now(),

    `updated_at` DateTime DEFAULT now(),

    `metadata` String
)
ENGINE = ReplacingMergeTree
PARTITION BY dataset_id
ORDER BY (id,
 topic_id,
 dataset_id,
 created_at)
SETTINGS index_granularity = 8192;