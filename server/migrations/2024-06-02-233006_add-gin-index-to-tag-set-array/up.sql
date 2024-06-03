-- Your SQL goes here
CREATE INDEX tag_set_array_idx ON chunk_metadata USING GIN (tag_set_array);

