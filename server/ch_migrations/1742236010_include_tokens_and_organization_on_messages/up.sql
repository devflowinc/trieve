ALTER TABLE search_queries ADD COLUMN IF NOT EXISTS tokens UInt64;
ALTER TABLE search_queries ADD COLUMN IF NOT EXISTS organization_id String;

ALTER TABLE rag_queries ADD COLUMN IF NOT EXISTS tokens UInt64;
ALTER TABLE rag_queries ADD COLUMN IF NOT EXISTS organization_id String;

ALTER TABLE recommendations ADD COLUMN IF NOT EXISTS organization_id String;
