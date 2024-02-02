-- This file should undo anything in `up.sql`
ALTER TABLE messages
DROP CONSTRAINT IF EXISTS messages_topic_id_fkey;

ALTER TABLE messages
ADD CONSTRAINT messages_topic_id_fkey
FOREIGN KEY (topic_id) REFERENCES topics(id);

