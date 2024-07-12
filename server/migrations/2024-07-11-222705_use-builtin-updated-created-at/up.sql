-- Your SQL goes here

-- Drop the triggers
DROP TRIGGER IF EXISTS update_updated_at ON chunk_group;
DROP TRIGGER IF EXISTS update_updated_at ON chunk_group_bookmarks;
DROP TRIGGER IF EXISTS update_updated_at ON chunk_metadata;
DROP TRIGGER IF EXISTS update_updated_at ON files;
DROP TRIGGER IF EXISTS update_updated_at ON groups_from_files;
DROP TRIGGER IF EXISTS update_updated_at ON messages;
DROP TRIGGER IF EXISTS update_updated_at ON topics;
DROP TRIGGER IF EXISTS update_updated_at ON users;

-- Drop the functions
DROP FUNCTION IF EXISTS update_updated_at();
DROP FUNCTION IF EXISTS update_main_table_updated_at();

CREATE OR REPLACE FUNCTION diesel_manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION diesel_set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

SELECT diesel_manage_updated_at('chunk_group');
SELECT diesel_manage_updated_at('chunk_group_bookmarks');
SELECT diesel_manage_updated_at('chunk_metadata');
SELECT diesel_manage_updated_at('files');
SELECT diesel_manage_updated_at('groups_from_files');
SELECT diesel_manage_updated_at('messages');
SELECT diesel_manage_updated_at('topics');
SELECT diesel_manage_updated_at('users');
