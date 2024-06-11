-- This file should undo anything in `up.sql`
ALTER TABLE dataset_tags
ADD CONSTRAINT dataset_tags_tag_key UNIQUE(tag);
