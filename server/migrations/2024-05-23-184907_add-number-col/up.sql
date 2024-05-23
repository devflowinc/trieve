-- Your SQL goes here
ALTER TABLE chunk_metadata ADD COLUMN num_value float8;
CREATE INDEX idx_num_val_chunk_metadata ON chunk_metadata USING btree (num_value);
