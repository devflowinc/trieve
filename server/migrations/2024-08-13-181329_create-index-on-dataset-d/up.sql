-- Your SQL goes here
-- Index on dataset_tags for faster filtering
CREATE INDEX idx_dataset_tags_dataset_id ON dataset_tags(dataset_id);
