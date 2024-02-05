-- Your SQL goes here
ALTER TABLE chunk_group_bookmarks
  DROP CONSTRAINT card_collection_bookmarks_card_metadata_id_fkey;

ALTER TABLE chunk_group_bookmarks
  ADD CONSTRAINT chunk_group_bookmarks_chunk_metadata_id_fkey
  FOREIGN KEY (chunk_metadata_id) REFERENCES chunk_metadata(id) ON DELETE CASCADE ON UPDATE CASCADE;
