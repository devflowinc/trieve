-- Your SQL goes here
ALTER TABLE chunk_group ADD COLUMN metadata JSONB NULL DEFAULT '{}'::JSONB;
ALTER TABLE chunk_group ADD COLUMN tag_set text NULL;

CREATE INDEX idx_chunk_group_metadata ON chunk_group(metadata);
CREATE INDEX idx_chunk_group_tag_set ON chunk_group(tag_set);