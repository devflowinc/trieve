-- Your SQL goes here
ALTER TABLE
    files
ADD
    COLUMN dataset_id UUID NOT NULL;

ALTER TABLE
    files
ADD
    CONSTRAINT files_dataset_id_fkey FOREIGN KEY (dataset_id) REFERENCES datasets(id);