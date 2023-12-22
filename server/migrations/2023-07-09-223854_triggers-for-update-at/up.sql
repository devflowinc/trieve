-- Your SQL goes here
ALTER TABLE chunk_verification ADD COLUMN created_at TIMESTAMP NOT NULL DEFAULT NOW();
ALTER TABLE chunk_verification ADD COLUMN updated_at TIMESTAMP NOT NULL DEFAULT NOW();


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
BEFORE UPDATE ON chunk_collection
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON chunk_collection_bookmarks
FOR EACH ROW
EXECUTE FUNCTION update_main_table_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON chunk_collisions
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON chunk_files
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON chunk_metadata
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON chunk_verification
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON chunk_votes
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON collections_from_files
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON files
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON invitations
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON messages
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON password_resets
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON stripe_customers
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON topics
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON user_plans
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_updated_at
BEFORE UPDATE ON verification_notifications
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();