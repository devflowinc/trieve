-- This file should undo anything in `up.sql`

-- Drop the triggers created by diesel_manage_updated_at
DROP TRIGGER IF EXISTS set_updated_at ON chunk_group;
DROP TRIGGER IF EXISTS set_updated_at ON chunk_group_bookmarks;
DROP TRIGGER IF EXISTS set_updated_at ON chunk_metadata;
DROP TRIGGER IF EXISTS set_updated_at ON files;
DROP TRIGGER IF EXISTS set_updated_at ON groups_from_files;
DROP TRIGGER IF EXISTS set_updated_at ON messages;
DROP TRIGGER IF EXISTS set_updated_at ON topics;
DROP TRIGGER IF EXISTS set_updated_at ON users;

-- Drop the functions created in the up migration
DROP FUNCTION IF EXISTS diesel_manage_updated_at(regclass);
DROP FUNCTION IF EXISTS diesel_set_updated_at();

CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = current_timestamp;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_main_table_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  UPDATE main_table SET updated_at = current_timestamp WHERE id = NEW.collection_id;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON chunk_group
FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON chunk_group_bookmarks
FOR EACH ROW EXECUTE FUNCTION update_main_table_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON chunk_metadata
FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON files
FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON groups_from_files
FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON messages
FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON topics
FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION update_updated_at();
