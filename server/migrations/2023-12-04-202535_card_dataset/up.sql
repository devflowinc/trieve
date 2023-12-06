CREATE TABLE datasets (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO datasets (name) VALUES ('DEFAULT');

ALTER TABLE card_metadata
ADD COLUMN dataset_id UUID NULL;

ALTER TABLE card_collection
ADD COLUMN dataset_id UUID NULL;

ALTER TABLE card_collection_bookmarks
ADD COLUMN dataset_id UUID NULL;

UPDATE card_metadata
SET dataset_id = datasets.id
FROM datasets
WHERE
  datasets.name = 'DEFAULT';

UPDATE card_collection
SET dataset_id = datasets.id
FROM datasets
WHERE datasets.name = 'DEFAULT';

UPDATE card_collection_bookmarks
SET dataset_id = datasets.id
FROM datasets
WHERE datasets.name = 'DEFAULT';

ALTER TABLE card_metadata ALTER COLUMN dataset_id SET NOT NULL;
ALTER TABLE card_collection ALTER COLUMN dataset_id SET NOT NULL;
ALTER TABLE card_collection_bookmarks ALTER COLUMN dataset_id SET NOT NULL;
