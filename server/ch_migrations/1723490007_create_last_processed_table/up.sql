CREATE TABLE IF NOT EXISTS dataset_words_last_processed (
	last_processed DateTime DEFAULT now() NOT NULL,
	dataset_id UUID NOT NULL,
) ENGINE = ReplacingMergeTree(last_processed)
ORDER BY (dataset_id)
PARTITION BY dataset_id;
