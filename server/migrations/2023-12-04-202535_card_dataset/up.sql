CREATE TABLE datasets (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

ALTER TABLE card_metadata ALTER COLUMN dataset_id SET NOT NULL;
ALTER TABLE card_collection ALTER COLUMN dataset_id SET NOT NULL;
ALTER TABLE card_collection_bookmarks ALTER COLUMN dataset_id SET NOT NULL;
