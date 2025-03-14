ALTER TABLE search_queries ADD COLUMN IF NOT EXISTS metadata String;
ALTER TABLE rag_queries ADD COLUMN IF NOT EXISTS metadata String;
ALTER TABLE recommendations ADD COLUMN IF NOT EXISTS metadata String;
ALTER TABLE topics ADD COLUMN IF NOT EXISTS metadata String;
ALTER TABLE topics DROP COLUMN IF EXISTS referrer;