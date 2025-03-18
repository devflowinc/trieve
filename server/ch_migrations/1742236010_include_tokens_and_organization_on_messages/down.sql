ALTER TABLE search_queries DROP COLUMN IF EXISTS tokens;
ALTER TABLE search_queries DROP COLUMN IF EXISTS organization_id;

ALTER TABLE rag_queries DROP COLUMN IF EXISTS tokens;
ALTER TABLE rag_queries DROP COLUMN IF EXISTS organization_id;

ALTER TABLE recommendations DROP COLUMN IF EXISTS organization_id;

ALTER TABLE dataset_events DROP COLUMN IF EXISTS organization_id;
