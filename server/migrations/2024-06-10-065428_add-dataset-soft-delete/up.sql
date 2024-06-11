-- Your SQL goes here
ALTER TABLE datasets ADD COLUMN deleted INT DEFAULT 0 NOT NULL;
CREATE INDEX IF NOT EXISTS idx_dataset_deleted ON datasets (deleted);