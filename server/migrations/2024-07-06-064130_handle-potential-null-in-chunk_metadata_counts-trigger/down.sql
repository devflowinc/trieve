-- This file should undo anything in `up.sql`
-- Your SQL goes here
DROP TRIGGER increase_chunk_metadata_counts_trigger ON chunk_metadata;
DROP TRIGGER delete_chunk_metadata_counts_trigger ON chunk_metadata;
DROP FUNCTION update_chunk_metadata_counts;

CREATE OR REPLACE FUNCTION update_chunk_metadata_counts()
RETURNS TRIGGER AS $$
DECLARE
    d_id UUID;
    new_count INT;
BEGIN
    SELECT dataset_id INTO d_id FROM modified LIMIT 1;
    SELECT COUNT(modified.id) INTO new_count FROM modified;

    IF TG_OP = 'INSERT' THEN
        INSERT INTO dataset_usage_counts (dataset_id, chunk_count)
        VALUES (d_id, new_count)
        ON CONFLICT (dataset_id) DO UPDATE
        SET chunk_count = dataset_usage_counts.chunk_count + new_count;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE dataset_usage_counts
        SET chunk_count = dataset_usage_counts.chunk_count - new_count
        WHERE dataset_id = d_id;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER increase_chunk_metadata_counts_trigger
AFTER INSERT ON chunk_metadata
REFERENCING NEW TABLE modified
FOR EACH STATEMENT
EXECUTE FUNCTION update_chunk_metadata_counts();

CREATE TRIGGER delete_chunk_metadata_counts_trigger
AFTER DELETE ON chunk_metadata
REFERENCING OLD TABLE modified
FOR EACH STATEMENT
EXECUTE FUNCTION update_chunk_metadata_counts();