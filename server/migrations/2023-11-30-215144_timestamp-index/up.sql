-- Your SQL goes here
CREATE INDEX card_time_stamp_index ON card_metadata(time_stamp);
CREATE INDEX file_time_stamp_index ON files(time_stamp);