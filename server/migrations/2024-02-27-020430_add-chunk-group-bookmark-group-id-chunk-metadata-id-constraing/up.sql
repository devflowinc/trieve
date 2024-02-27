-- Your SQL goes here
ALTER TABLE chunk_group_bookmarks
ADD CONSTRAINT group_id_chunk_metadata_id_key UNIQUE (group_id, chunk_metadata_id);