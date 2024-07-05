-- This file should undo anything in `up.sql`
DROP TRIGGER increase_chunk_metadata_counts_trigger ON chunk_metadata;
DROP TRIGGER delete_chunk_metadata_counts_trigger ON chunk_metadata;
DROP FUNCTION update_chunk_metadata_counts;

-- Function to update chunk metadata counts
CREATE OR REPLACE FUNCTION update_chunk_metadata_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Try to insert a new row into dataset_usage_counts
        INSERT INTO dataset_usage_counts (dataset_id, chunk_count)
        VALUES (NEW.dataset_id, 1)
        ON CONFLICT (dataset_id) DO UPDATE
        SET chunk_count = dataset_usage_counts.chunk_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement chunk_count when a chunk is deleted
        UPDATE dataset_usage_counts
        SET chunk_count = CASE WHEN dataset_usage_counts.chunk_count > 0 THEN dataset_usage_counts.chunk_count - 1 ELSE 0 END
        WHERE dataset_id = OLD.dataset_id;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger for chunk_metadata
CREATE OR REPLACE TRIGGER update_chunk_metadata_counts_trigger
AFTER INSERT OR DELETE ON chunk_metadata
FOR EACH ROW
EXECUTE FUNCTION update_chunk_metadata_counts();