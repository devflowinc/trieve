ALTER TABLE search_queries ADD COLUMN IF NOT EXISTS tokens UInt64;
ALTER TABLE search_queries ADD COLUMN IF NOT EXISTS organization_id UUID;

ALTER TABLE rag_queries ADD COLUMN IF NOT EXISTS tokens UInt64;
ALTER TABLE rag_queries ADD COLUMN IF NOT EXISTS organization_id UUID;

ALTER TABLE recommendations ADD COLUMN IF NOT EXISTS organization_id UUID;

ALTER TABLE dataset_events ADD COLUMN IF NOT EXISTS organization_id Nullable(UUID);
