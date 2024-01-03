
CREATE TABLE IF NOT EXISTS organization_usage_counts (
    id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    dataset_count INTEGER NOT NULL DEFAULT 0,    
    user_count INTEGER NOT NULL DEFAULT 0,
    file_storage INTEGER NOT NULL DEFAULT 0,
    message_count INTEGER NOT NULL DEFAULT 0
);
-- Create table for dataset usage counts
CREATE TABLE IF NOT EXISTS dataset_usage_counts (
    id UUID PRIMARY KEY REFERENCES datasets(id) ON DELETE CASCADE,
    chunk_count INTEGER NOT NULL DEFAULT 0
);

-- Function to update chunk metadata counts
CREATE OR REPLACE FUNCTION update_chunk_metadata_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Try to insert a new row into dataset_usage_counts
        INSERT INTO dataset_usage_counts (id, chunk_count)
        VALUES (NEW.dataset_id, 1)
        ON CONFLICT (id) DO UPDATE
        SET chunk_count = dataset_usage_counts.chunk_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement chunk_count when a chunk is deleted
        UPDATE dataset_usage_counts
        SET chunk_count = CASE WHEN dataset_usage_counts.chunk_count > 0 THEN dataset_usage_counts.chunk_count - 1 ELSE 0 END
        WHERE id = OLD.dataset_id;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger for chunk_metadata
CREATE OR REPLACE TRIGGER update_chunk_metadata_counts_trigger
AFTER INSERT OR DELETE ON chunk_metadata
FOR EACH ROW
EXECUTE FUNCTION update_chunk_metadata_counts();

-- Function to update files storage counts
CREATE OR REPLACE FUNCTION update_files_storage_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Update file_storage for new files or insert a new row if the organization doesn't exist
        INSERT INTO organization_usage_counts (id, file_storage)
        VALUES ((SELECT organization_id FROM datasets WHERE id = NEW.dataset_id), NEW.size)
        ON CONFLICT (id) DO UPDATE
        SET file_storage = organization_usage_counts.file_storage + NEW.size;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement file_storage when a file is deleted
        UPDATE organization_usage_counts
        SET file_storage = CASE WHEN organization_usage_counts.file_storage > OLD.size THEN organization_usage_counts.file_storage - OLD.size ELSE 0 END
        WHERE id = (SELECT organization_id FROM datasets WHERE id = OLD.dataset_id);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for files
CREATE OR REPLACE TRIGGER update_files_storage_trigger
AFTER INSERT OR DELETE ON files
FOR EACH ROW
EXECUTE FUNCTION update_files_storage_counts();

-- Function to update messages counts
CREATE OR REPLACE FUNCTION update_messages_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Update message_count for new messages or insert a new row if the organization doesn't exist
        INSERT INTO organization_usage_counts (id, message_count)
        VALUES ((SELECT organization_id FROM datasets WHERE id = OLD.dataset_id), 1)
        ON CONFLICT (id) DO UPDATE
        SET message_count = organization_usage_counts.message_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement message_count when a message is deleted
        UPDATE organization_usage_counts
        SET message_count = CASE WHEN organization_usage_counts.message_count > 0 THEN organization_usage_counts.message_count - 1 ELSE 0 END
        WHERE id = (SELECT organization_id FROM datasets WHERE id = OLD.dataset_id);
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
        INSERT INTO organization_usage_counts (id, dataset_count)
        VALUES (NEW.organization_id, 1)
        ON CONFLICT (id) DO UPDATE
        SET dataset_count = organization_usage_counts.dataset_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement dataset_count when a dataset is deleted
        UPDATE organization_usage_counts
        SET dataset_count = CASE WHEN organization_usage_counts.dataset_count > 0 THEN organization_usage_counts.dataset_count - 1 ELSE 0 END
        WHERE id = OLD.organization_id;
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
        INSERT INTO organization_usage_counts (id, user_count)
        VALUES (NEW.organization_id, 1)
        ON CONFLICT (id) DO UPDATE
        SET user_count = organization_usage_counts.user_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement user_count when a user is deleted
        UPDATE organization_usage_counts
        SET user_count = CASE WHEN organization_usage_counts.user_count > 0 THEN organization_usage_counts.user_count - 1 ELSE 0 END
        WHERE id = OLD.organization_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for users
CREATE OR REPLACE TRIGGER update_users_counts_trigger
AFTER INSERT OR DELETE ON users
FOR EACH ROW
EXECUTE FUNCTION update_users_counts();

DROP TRIGGER IF EXISTS card_metadata_count_trigger ON chunk_metadata;
DROP TABLE IF EXISTS chunk_metadata_counts;