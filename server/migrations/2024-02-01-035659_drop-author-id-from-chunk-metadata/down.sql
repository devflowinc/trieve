-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata
ADD COLUMN author_id UUID NOT NULL REFERENCES users(id);

ALTER TABLE user_organizations ADD CONSTRAINT fk_user_id FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;

ALTER TABLE user_api_key ADD CONSTRAINT user_api_key_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE files ADD CONSTRAINT files_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE topics ADD CONSTRAINT topics_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
