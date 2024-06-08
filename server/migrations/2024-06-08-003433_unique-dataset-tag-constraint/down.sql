-- This file should undo anything in `up.sql`
ALTER TABLE dataset_tags
DROP CONSTRAINT IF EXISTS dataset_tags_dataset_id_tag_key;

ALTER TABLE dataset_tags
ADD CONSTRAINT dataset_tags_id_dataset_id UNIQUE(id, dataset_id);

ALTER TABLE dataset_tags
ADD CONSTRAINT dataset_tags_tag UNIQUE(tag);
