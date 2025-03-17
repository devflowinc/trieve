ALTER TABLE search_queries DROP COLUMN IF NOT EXISTS tokens;
ALTER TABLE search_queries DROP COLUMN IF NOT EXISTS organization_id;

ALTER TABLE rag_queries DROP COLUMN IF NOT EXISTS tokens;
ALTER TABLE rag_queries DROP COLUMN IF NOT EXISTS organization_id;

ALTER TABLE recommendations DROP COLUMN IF NOT EXISTS organization_id;
