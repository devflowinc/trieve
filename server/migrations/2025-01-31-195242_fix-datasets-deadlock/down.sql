-- Create or replace the function to update organization chunk count
CREATE OR REPLACE FUNCTION update_organization_chunk_count() RETURNS TRIGGER AS $$
BEGIN
    UPDATE organization_usage_counts o
    SET chunk_count = (
        SELECT COALESCE(SUM(duc.chunk_count), 0)
        FROM dataset_usage_counts duc
        JOIN datasets d ON d.id = duc.dataset_id
        WHERE d.organization_id = o.org_id
    )
    WHERE o.org_id IN (
        SELECT DISTINCT d.organization_id
        FROM datasets d
        WHERE d.id IN (
            SELECT dataset_id 
            FROM dataset_usage_counts 
            WHERE (TG_OP = 'INSERT' OR TG_OP = 'UPDATE' OR TG_OP = 'DELETE')
        )
    );
    
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create the trigger to call the function after insert, update, or delete on dataset_usage_counts
CREATE OR REPLACE TRIGGER update_organization_chunk_count_trigger
AFTER INSERT OR UPDATE OR DELETE ON dataset_usage_counts
FOR EACH STATEMENT
EXECUTE FUNCTION update_organization_chunk_count();

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

CREATE OR REPLACE TRIGGER increase_chunk_metadata_counts_trigger
AFTER INSERT ON chunk_metadata
REFERENCING NEW TABLE modified
FOR EACH STATEMENT
EXECUTE FUNCTION update_chunk_metadata_counts();

CREATE OR REPLACE TRIGGER delete_chunk_metadata_counts_trigger
AFTER DELETE ON chunk_metadata
REFERENCING OLD TABLE modified
FOR EACH STATEMENT
EXECUTE FUNCTION update_chunk_metadata_counts();
