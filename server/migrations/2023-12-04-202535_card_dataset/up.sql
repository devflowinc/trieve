CREATE TABLE datasets (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

ALTER TABLE card_metadata ADD COLUMN dataset_id UUID NOT NULL;
ALTER TABLE card_collection ADD COLUMN dataset_id UUID NOT NULL;
ALTER TABLE card_collection_bookmarks ADD COLUMN dataset_id UUID NOT NULL;
