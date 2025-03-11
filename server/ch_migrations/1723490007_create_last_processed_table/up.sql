CREATE TABLE IF NOT EXISTS dataset_words_last_processed on CLUSTER `{cluster}`
(
	last_processed DateTime DEFAULT now() NOT NULL,
	dataset_id UUID NOT NULL,
) ENGINE = ReplicatedReplacingMergeTree(last_processed)
ORDER BY (dataset_id)
PARTITION BY dataset_id;
