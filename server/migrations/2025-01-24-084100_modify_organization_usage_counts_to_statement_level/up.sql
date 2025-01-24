-- Your SQL goes here
CREATE OR REPLACE FUNCTION update_messages_counts()
RETURNS TRIGGER AS $$
DECLARE
    changed_dataset_id UUID;
    changed_organization_id UUID;
    new_messages INT;
BEGIN
    SELECT dataset_id INTO changed_dataset_id FROM modified LIMIT 1;
    SELECT COUNT(modified.id) INTO new_messages FROM modified;
    SELECT organization_id INTO changed_organization_id FROM datasets WHERE id = changed_dataset_id;

    INSERT INTO organization_usage_counts (org_id, message_count)
    VALUES (changed_organization_id, new_messages)
    ON CONFLICT (org_id) DO UPDATE
    SET message_count = organization_usage_counts.message_count + new_messages;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER update_messages_counts_trigger ON messages;

CREATE OR REPLACE TRIGGER update_messages_counts_trigger
AFTER INSERT ON messages
REFERENCING NEW TABLE modified
FOR EACH STATEMENT
EXECUTE FUNCTION update_messages_counts();

-- Function to update datasets counts
CREATE OR REPLACE FUNCTION update_datasets_counts()
RETURNS TRIGGER AS $$
DECLARE
    dataset_organization_id UUID;
    amount_changed INT;
BEGIN
    SELECT organization_id INTO dataset_organization_id FROM modified LIMIT 1;
    RAISE LOG 'this is getting called';
    SELECT COUNT(modified.id) INTO amount_changed FROM modified;

    IF TG_OP = 'INSERT' THEN
        -- Update dataset_count for new datasets or insert a new row if the organization doesn't exist
        INSERT INTO organization_usage_counts (org_id, dataset_count)
        VALUES (dataset_organization_id, amount_changed)
        ON CONFLICT (org_id) DO UPDATE
        SET dataset_count = organization_usage_counts.dataset_count + amount_changed;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement dataset_count when a dataset is deleted
        UPDATE organization_usage_counts
        SET dataset_count = organization_usage_counts.dataset_count - amount_changed
        WHERE org_id = dataset_organization_id;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger for datasets
CREATE OR REPLACE TRIGGER update_datasets_counts_trigger
AFTER INSERT ON datasets
REFERENCING NEW TABLE modified
FOR EACH STATEMENT
EXECUTE FUNCTION update_datasets_counts();

CREATE OR REPLACE TRIGGER delete_datasets_counts_trigger
AFTER DELETE ON datasets
REFERENCING OLD TABLE modified
FOR EACH STATEMENT
EXECUTE FUNCTION update_datasets_counts();

CREATE OR REPLACE FUNCTION update_files_storage_counts_with_update()
RETURNS TRIGGER AS $$
DECLARE
    changed_dataset_id UUID;
    changed_organization_id UUID;
    delta_increased BIGINT;
    delta_removed BIGINT;
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Update file_storage for new files or insert a new row if the organization doesn't exist
        SELECT dataset_id INTO changed_dataset_id FROM new_files LIMIT 1;
        SELECT SUM(size) INTO delta_increased FROM new_files;
        SELECT organization_id INTO changed_organization_id FROM datasets WHERE id = changed_dataset_id;

        INSERT INTO organization_usage_counts (org_id, file_storage)
        VALUES (changed_organization_id, delta_increased)
        ON CONFLICT (org_id) DO UPDATE
        SET file_storage = organization_usage_counts.file_storage + delta_increased;
    ELSIF TG_OP = 'UPDATE' THEN
        SELECT dataset_id INTO changed_dataset_id FROM old_files LIMIT 1;
        SELECT SUM(size) INTO delta_increased FROM new_files;
        SELECT SUM(size) INTO delta_removed FROM old_files;
        SELECT organization_id INTO changed_organization_id FROM datasets WHERE id = changed_dataset_id;

        UPDATE organization_usage_counts
        SET file_storage = GREATEST(0, organization_usage_counts.file_storage - delta_removed + delta_increased)
        WHERE org_id = changed_organization_id;
    ELSIF TG_OP = 'DELETE' THEN
        -- Update file_storage for new files or insert a new row if the organization doesn't exist
        SELECT dataset_id INTO changed_dataset_id FROM old_files LIMIT 1;
        SELECT SUM(size) INTO delta_removed FROM old_files;
        SELECT organization_id INTO changed_organization_id FROM datasets WHERE id = changed_dataset_id;

        -- Decrement file_storage when a file is deleted
        UPDATE organization_usage_counts
        SET file_storage = organization_usage_counts.file_storage - delta_removed
        WHERE org_id = changed_organization_id;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;


CREATE OR REPLACE TRIGGER insert_files_storage_with_update_trigger
AFTER INSERT ON files
REFERENCING NEW TABLE new_files
FOR EACH STATEMENT
EXECUTE FUNCTION update_files_storage_counts_with_update();

-- Create a new trigger that includes the 'UPDATE' operation
CREATE OR REPLACE TRIGGER update_files_storage_with_update_trigger
AFTER UPDATE ON files
REFERENCING NEW TABLE as new_files OLD TABLE as old_files
FOR EACH STATEMENT
EXECUTE FUNCTION update_files_storage_counts_with_update();

-- Create a new trigger that includes the 'UPDATE' operation
CREATE OR REPLACE TRIGGER update_files_storage_with_update_trigger
AFTER DELETE ON files
REFERENCING OLD TABLE as old_files
FOR EACH STATEMENT
EXECUTE FUNCTION update_files_storage_counts_with_update();


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

