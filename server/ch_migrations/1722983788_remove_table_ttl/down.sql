ALTER TABLE dataset_events MODIFY TTL created_at + INTERVAL 30 DAY;
ALTER TABLE search_queries MODIFY TTL created_at + INTERVAL 30 DAY;
ALTER TABLE rag_queries MODIFY TTL created_at + INTERVAL 30 DAY;
ALTER TABLE recommendations MODIFY TTL created_at + INTERVAL 30 DAY;
ALTER TABLE ctr_data MODIFY TTL created_at + INTERVAL 30 DAY; 
