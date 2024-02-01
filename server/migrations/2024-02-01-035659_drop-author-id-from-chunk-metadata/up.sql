-- Your SQL goes here
ALTER TABLE chunk_metadata
DROP COLUMN author_id;

-- Drop the existing foreign key constraint from user_organizations
ALTER TABLE user_organizations
DROP CONSTRAINT IF EXISTS fk_user_id;

-- Create a new foreign key constraint with ON UPDATE CASCADE on user_organizations
ALTER TABLE user_organizations
ADD CONSTRAINT fk_user_id
FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE user_api_key
DROP CONSTRAINT user_api_key_user_id_fkey;

ALTER TABLE user_api_key
ADD CONSTRAINT user_api_key_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE files
DROP CONSTRAINT files_user_id_fkey;

ALTER TABLE files
ADD CONSTRAINT files_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE topics
DROP CONSTRAINT topics_user_id_fkey;

ALTER TABLE topics
ADD CONSTRAINT topics_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE;



