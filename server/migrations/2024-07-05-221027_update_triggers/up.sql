-- Your SQL goes here
DROP TRIGGER IF EXISTS update_chunk_metadata_counts_trigger ON chunk_metadata;
DROP FUNCTION IF EXISTS update_chunk_metadata_counts();

-- Function for INSERT operations
CREATE OR REPLACE FUNCTION update_chunk_metadata_count_insert() 
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO dataset_usage_counts (dataset_id, chunk_count)
    SELECT dataset_id, COUNT(*)
    FROM inserted
    GROUP BY dataset_id
    ON CONFLICT (dataset_id) DO UPDATE
    SET chunk_count = dataset_usage_counts.chunk_count + EXCLUDED.chunk_count;
    
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function for DELETE operations
CREATE OR REPLACE FUNCTION update_chunk_metadata_count_delete() 
RETURNS TRIGGER AS $$
BEGIN
    UPDATE dataset_usage_counts
    SET chunk_count = dataset_usage_counts.chunk_count - subquery.count
    FROM (
        SELECT dataset_id, COUNT(*) as count
        FROM deleted
        GROUP BY dataset_id
    ) as subquery
    WHERE dataset_usage_counts.dataset_id = subquery.dataset_id;
    
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger for INSERT operations
CREATE TRIGGER update_chunk_metadata_counts_trigger_insert
AFTER INSERT ON chunk_metadata
REFERENCING NEW TABLE AS inserted
FOR EACH STATEMENT
EXECUTE FUNCTION update_chunk_metadata_count_insert();

-- Trigger for DELETE operations
CREATE TRIGGER update_chunk_metadata_counts_trigger_delete
AFTER DELETE ON chunk_metadata
REFERENCING OLD TABLE AS deleted
FOR EACH STATEMENT
EXECUTE FUNCTION update_chunk_metadata_count_delete();
