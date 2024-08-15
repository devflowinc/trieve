ALTER TABLE search_queries ADD COLUMN user_id String;
ALTER TABLE recommendations ADD COLUMN user_id String;
ALTER TABLE rag_queries ADD COLUMN user_id String;