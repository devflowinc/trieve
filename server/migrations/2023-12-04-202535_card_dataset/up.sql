CREATE TABLE dataset (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO dataset (name) VALUES ('DEFAULT');

CREATE OR REPLACE FUNCTION set_default_dataset_id()
RETURNS TRIGGER AS $$
BEGIN
  IF NEW.dataset_id IS NULL THEN
    SELECT id INTO NEW.dataset_id FROM dataset WHERE name = 'DEFAULT';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TRIGGER set_default_dataset_id_trigger
BEFORE INSERT ON card_metadata
FOR EACH ROW EXECUTE FUNCTION set_default_dataset_id();

CREATE TRIGGER set_default_dataset_id_trigger
BEFORE INSERT ON card_collection
FOR EACH ROW EXECUTE FUNCTION set_default_dataset_id();

CREATE TRIGGER set_default_dataset_id_trigger
BEFORE INSERT ON card_collection_bookmarks
FOR EACH ROW EXECUTE FUNCTION set_default_dataset_id();

ALTER TABLE card_metadata
ADD COLUMN dataset_id UUID NOT NULL;

ALTER TABLE card_collection
ADD COLUMN dataset_id UUID NOT NULL;

ALTER TABLE card_collection_bookmarks
ADD COLUMN dataset_id UUID NOT NULL;
