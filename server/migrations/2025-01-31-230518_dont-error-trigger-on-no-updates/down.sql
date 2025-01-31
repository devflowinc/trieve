-- This file should undo anything in `up.sql`
CREATE OR REPLACE FUNCTION update_chunk_metadata_counts()
RETURNS TRIGGER AS $$
DECLARE
    changed_dataset_id UUID;
    changed_organization_id UUID;
    chunks_created INT;
BEGIN

    IF TG_OP = 'INSERT' THEN
        SELECT dataset_id INTO changed_dataset_id FROM new_table LIMIT 1;
        SELECT COUNT(new_table.id) INTO chunks_created FROM new_table;
        SELECT organization_id INTO changed_organization_id FROM datasets WHERE id = changed_dataset_id;

        INSERT INTO dataset_usage_counts (dataset_id, chunk_count)
        VALUES (changed_dataset_id, chunks_created)
        ON CONFLICT (dataset_id) DO UPDATE
        SET chunk_count = dataset_usage_counts.chunk_count + chunks_created;

        INSERT INTO organization_usage_counts (org_id, chunk_count)
        VALUES (changed_organization_id, chunks_created)
        ON CONFLICT (org_id) DO UPDATE
        SET chunk_count = organization_usage_counts.chunk_count + chunks_created;

    ELSIF TG_OP = 'DELETE' THEN
        SELECT dataset_id INTO changed_dataset_id FROM old_table LIMIT 1;
        SELECT organization_id INTO changed_organization_id FROM datasets WHERE id = changed_dataset_id;
        SELECT COUNT(old_table.id) INTO chunks_created FROM old_table;

        UPDATE dataset_usage_counts
        SET chunk_count = dataset_usage_counts.chunk_count - chunks_created
        WHERE dataset_id = changed_dataset_id;

        UPDATE organization_usage_counts
        SET chunk_count = organization_usage_counts.chunk_count - chunks_created
        WHERE org_id = changed_organization_id;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER increase_chunk_metadata_counts_trigger
AFTER INSERT ON chunk_metadata
REFERENCING NEW TABLE new_table
FOR EACH STATEMENT
EXECUTE FUNCTION update_chunk_metadata_counts();

CREATE OR REPLACE TRIGGER delete_chunk_metadata_counts_trigger
AFTER DELETE ON chunk_metadata
REFERENCING OLD TABLE old_table
FOR EACH STATEMENT
EXECUTE FUNCTION update_chunk_metadata_counts();
