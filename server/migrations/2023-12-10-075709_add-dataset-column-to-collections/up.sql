-- Your SQL goes here
ALTER TABLE
    chunk_collection_bookmarks DROP COLUMN IF EXISTS dataset_id;

ALTER TABLE
    chunk_metadata
ADD
    COLUMN dataset_id UUID NOT NULL;

ALTER TABLE
    chunk_collection
ADD
    COLUMN dataset_id UUID NOT NULL;

ALTER TABLE
    chunk_metadata
ADD
    CONSTRAINT chunk_metadata_dataset_id_fkey FOREIGN KEY (dataset_id) REFERENCES datasets(id);

ALTER TABLE
    chunk_collection
ADD
    CONSTRAINT chunk_collection_dataset_id_fkey FOREIGN KEY (dataset_id) REFERENCES datasets(id);