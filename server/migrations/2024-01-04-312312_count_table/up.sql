-- Function to update files storage counts
CREATE OR REPLACE FUNCTION update_files_storage_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
      -- Update file storage for new files or insert a new row if the organization doesn't exist
      INSERT INTO organization_usage_counts (id, file_storage)
      VALUES ((SELECT organization_id FROM datasets WHERE id = NEW.dataset_id), NEW.size)
      ON CONFLICT (id) DO UPDATE
      SET file_storage = organization_usage_counts.file_storage + (NEW.size - COALESCE(OLD.size, 0));
    ELSIF TG_OP = 'DELETE' THEN
      -- Decrement file storage when a file is deleted
      UPDATE organization_usage_counts
      SET file_storage = CASE WHEN organization_usage_counts.file_storage > OLD.size THEN organization_usage_counts.file_storage - OLD.size ELSE 0 END
      WHERE id = (SELECT organization_id FROM datasets WHERE id = OLD.dataset_id);
    ELSIF TG_OP = 'UPDATE' THEN
      -- Update file storage when the file size changes
      UPDATE organization_usage_counts
      SET file_storage = CASE WHEN organization_usage_counts.file_storage > (NEW.size - OLD.size) THEN organization_usage_counts.file_storage + (NEW.size - OLD.size) ELSE 0 END
      WHERE id = (SELECT organization_id FROM datasets WHERE id = NEW.dataset_id);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for files
CREATE OR REPLACE TRIGGER update_files_storage_trigger
AFTER INSERT OR UPDATE OR DELETE ON files
FOR EACH ROW
EXECUTE FUNCTION update_files_storage_counts();