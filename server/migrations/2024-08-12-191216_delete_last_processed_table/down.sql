-- This file should undo anything in `up.sql`
CREATE TABLE IF NOT EXISTS "dataset_words_last_processed" (
	id UUID PRIMARY KEY,
	last_processed TIMESTAMP NULL,
	dataset_id UUID NOT NULL,
	FOREIGN KEY (dataset_id) REFERENCES "datasets"(id) ON DELETE CASCADE,
	UNIQUE(dataset_id)
);
