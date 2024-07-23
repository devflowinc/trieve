ALTER TABLE search_queries
ADD COLUMN IF NOT EXISTS is_duplicate UInt8 DEFAULT 0;
