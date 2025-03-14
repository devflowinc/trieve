ALTER TABLE search_queries DROP COLUMN IF EXISTS metadata;
ALTER TABLE rag_queries DROP COLUMN IF EXISTS metadata;
ALTER TABLE recommendations DROP COLUMN IF EXISTS metadata;
ALTER TABLE topics DROP COLUMN IF EXISTS metadata;
ALTER TABLE topics ADD COLUMN IF NOT EXISTS referrer String;