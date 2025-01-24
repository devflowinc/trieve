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

-- Create a new trigger that includes the 'UPDATE' operation
CREATE OR REPLACE TRIGGER update_files_storage_with_update_trigger
AFTER INSERT OR UPDATE OR DELETE ON files
FOR EACH ROW
EXECUTE FUNCTION update_files_storage_counts_with_update();

-- Function to update messages counts
CREATE OR REPLACE FUNCTION update_messages_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Update message_count for new messages or insert a new row if the organization doesn't exist
        INSERT INTO organization_usage_counts (org_id, message_count)
        VALUES ((SELECT organization_id FROM datasets WHERE id = OLD.dataset_id), 1)
        ON CONFLICT (org_id) DO UPDATE
        SET message_count = organization_usage_counts.message_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement message_count when a message is deleted
        UPDATE organization_usage_counts
        SET message_count = CASE WHEN organization_usage_counts.message_count > 0 THEN organization_usage_counts.message_count - 1 ELSE 0 END
        WHERE org_id = (SELECT organization_id FROM datasets WHERE id = OLD.dataset_id);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for messages
CREATE OR REPLACE TRIGGER update_messages_counts_trigger
AFTER INSERT OR DELETE ON messages
FOR EACH ROW
EXECUTE FUNCTION update_messages_counts();

-- Function to update datasets counts
CREATE OR REPLACE FUNCTION update_datasets_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Update dataset_count for new datasets or insert a new row if the organization doesn't exist
        INSERT INTO organization_usage_counts (org_id, dataset_count)
        VALUES (NEW.organization_id, 1)
        ON CONFLICT (org_id) DO UPDATE
        SET dataset_count = organization_usage_counts.dataset_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement dataset_count when a dataset is deleted
        UPDATE organization_usage_counts
        SET dataset_count = CASE WHEN organization_usage_counts.dataset_count > 0 THEN organization_usage_counts.dataset_count - 1 ELSE 0 END
        WHERE org_id = OLD.organization_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for datasets
CREATE OR REPLACE TRIGGER update_datasets_counts_trigger
AFTER INSERT OR DELETE ON datasets
FOR EACH ROW
EXECUTE FUNCTION update_datasets_counts();

CREATE OR REPLACE FUNCTION update_users_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Update user_count for new users or insert a new row if the organization doesn't exist
        INSERT INTO organization_usage_counts (org_id, user_count)
        VALUES (NEW.organization_id, 1)
        ON CONFLICT (org_id) DO UPDATE
        SET user_count = organization_usage_counts.user_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement user_count when a user is deleted
        UPDATE organization_usage_counts
        SET user_count = CASE WHEN organization_usage_counts.user_count > 0 THEN organization_usage_counts.user_count - 1 ELSE 0 END
        WHERE org_id = OLD.organization_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for users
CREATE OR REPLACE TRIGGER update_users_counts_trigger
AFTER INSERT OR DELETE ON user_organizations
FOR EACH ROW
EXECUTE FUNCTION update_users_counts();


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
CREATE OR REPLACE TRIGGER update_organization_chunk_count_trigger
AFTER INSERT OR UPDATE OR DELETE ON dataset_usage_counts
FOR EACH ROW
EXECUTE FUNCTION update_organization_chunk_count();
