-- Your SQL goes here

CREATE TABLE IF NOT EXISTS "words_in_datasets" (
	id UUID PRIMARY KEY,
	word TEXT NOT NULL,
	UNIQUE(word)
);

CREATE TABLE IF NOT EXISTS "words_datasets" (
	id UUID PRIMARY KEY,
	dataset_id UUID NOT NULL,
	word_id UUID NOT NULL,
	count INT NOT NULL,
	UNIQUE(dataset_id, word_id),
	FOREIGN KEY (dataset_id) REFERENCES "datasets"(id) ON DELETE CASCADE,
	FOREIGN KEY (word_id) REFERENCES "words_in_datasets"(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS "dataset_words_last_processed" (
	id UUID PRIMARY KEY,
	last_processed TIMESTAMP NULL,
	dataset_id UUID NOT NULL,
	FOREIGN KEY (dataset_id) REFERENCES "datasets"(id) ON DELETE CASCADE,
	UNIQUE(dataset_id)
);
