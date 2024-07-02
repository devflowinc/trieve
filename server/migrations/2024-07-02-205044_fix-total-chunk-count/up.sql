-- Update chunk_count to 0 where it is NULL
UPDATE organization_usage_counts
SET chunk_count = 0
WHERE chunk_count IS NULL;

-- Alter the column to set NOT NULL constraint
ALTER TABLE organization_usage_counts
ALTER COLUMN chunk_count SET NOT NULL;

-- Update chunk_count based on related dataset_usage_counts
UPDATE organization_usage_counts o
SET chunk_count = (
    SELECT COALESCE(SUM(duc.chunk_count), 0)
    FROM dataset_usage_counts duc
    JOIN datasets d ON d.id = duc.dataset_id
    WHERE d.organization_id = o.org_id
);

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
    WHERE o.org_id = (
        SELECT d.organization_id
        FROM datasets d
        WHERE d.id = NEW.dataset_id
    );
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create the trigger to call the function after insert, update, or delete on dataset_usage_counts
CREATE TRIGGER update_organization_chunk_count_trigger
AFTER INSERT OR UPDATE OR DELETE ON dataset_usage_counts
FOR EACH ROW
EXECUTE FUNCTION update_organization_chunk_count();
