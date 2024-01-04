CREATE OR REPLACE FUNCTION update_files_storage_counts_with_update()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Update file_storage for new files or insert a new row if the organization doesn't exist
        INSERT INTO organization_usage_counts (org_id, file_storage)
        VALUES ((SELECT organization_id FROM datasets WHERE id = NEW.dataset_id), NEW.size)
        ON CONFLICT (org_id) DO UPDATE
        SET file_storage = organization_usage_counts.file_storage + NEW.size;
    ELSIF TG_OP = 'UPDATE' THEN
        -- Update file_storage
        UPDATE organization_usage_counts
        SET file_storage = GREATEST(0, organization_usage_counts.file_storage - OLD.size + NEW.size)
        WHERE org_id = (SELECT organization_id FROM datasets WHERE id = NEW.dataset_id);
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement file_storage when a file is deleted
        UPDATE organization_usage_counts
        SET file_storage = CASE WHEN organization_usage_counts.file_storage > OLD.size THEN organization_usage_counts.file_storage - OLD.size ELSE 0 END
        WHERE org_id = (SELECT organization_id FROM datasets WHERE id = OLD.dataset_id);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Drop the old function if it exists
DROP FUNCTION IF EXISTS update_files_storage_counts();

-- Drop the old trigger if it exists
DROP TRIGGER IF EXISTS update_files_storage_trigger ON files;

-- Create a new trigger that includes the 'UPDATE' operation
CREATE TRIGGER update_files_storage_with_update_trigger
AFTER INSERT OR UPDATE OR DELETE ON files
FOR EACH ROW
EXECUTE FUNCTION update_files_storage_counts_with_update();
