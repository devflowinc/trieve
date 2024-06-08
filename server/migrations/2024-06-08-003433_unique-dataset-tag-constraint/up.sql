-- Your SQL goes here
ALTER TABLE dataset_tags
DROP CONSTRAINT IF EXISTS dataset_tags_tag;

ALTER TABLE dataset_tags
DROP CONSTRAINT IF EXISTS dataset_tags_id_dataset_id;

ALTER TABLE dataset_tags
ADD CONSTRAINT dataset_tags_dataset_id_tag_key UNIQUE (dataset_id, tag);
