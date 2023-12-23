-- Your SQL goes here
ALTER TABLE
    card_collection_bookmarks DROP COLUMN IF EXISTS dataset_id;

ALTER TABLE
    card_metadata
ADD
    COLUMN dataset_id UUID NOT NULL;

ALTER TABLE
    card_collection
ADD
    COLUMN dataset_id UUID NOT NULL;

ALTER TABLE
    card_metadata
ADD
    CONSTRAINT card_metadata_dataset_id_fkey FOREIGN KEY (dataset_id) REFERENCES datasets(id);

ALTER TABLE
    card_collection
ADD
    CONSTRAINT card_collection_dataset_id_fkey FOREIGN KEY (dataset_id) REFERENCES datasets(id);
