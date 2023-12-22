-- Your SQL goes here
CREATE INDEX chunk_time_stamp_index ON chunk_metadata(time_stamp);
CREATE INDEX file_time_stamp_index ON files(time_stamp);