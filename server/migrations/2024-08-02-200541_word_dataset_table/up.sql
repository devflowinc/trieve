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
	UNIQUE(dataset_id, word_id),
	FOREIGN KEY (dataset_id) REFERENCES "datasets"(id) ON DELETE CASCADE,
	FOREIGN KEY (word_id) REFERENCES "words_in_datasets"(id) ON DELETE CASCADE
);
