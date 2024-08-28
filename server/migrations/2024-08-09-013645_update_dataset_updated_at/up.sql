CREATE OR REPLACE FUNCTION update_chunk_metadata_counts()
RETURNS TRIGGER AS $$
DECLARE
    d_id UUID;
    new_count INT;
BEGIN
    SELECT dataset_id INTO d_id FROM modified WHERE dataset_id IS NOT NULL LIMIT 1;
    IF d_id IS NULL THEN
        RETURN NULL;
    END IF;
    SELECT COUNT(modified.id) INTO new_count FROM modified;

    IF TG_OP = 'INSERT' THEN
        -- Update dataset_usage_counts
        INSERT INTO dataset_usage_counts (dataset_id, chunk_count)
        VALUES (d_id, new_count)
        ON CONFLICT (dataset_id) DO UPDATE
        SET chunk_count = dataset_usage_counts.chunk_count + new_count;

        -- Update dataset
        UPDATE datasets
        SET updated_at = CURRENT_TIMESTAMP
        WHERE id = d_id;

    ELSIF TG_OP = 'DELETE' THEN
        -- Update dataset_usage_counts
        UPDATE dataset_usage_counts
        SET chunk_count = dataset_usage_counts.chunk_count - new_count
        WHERE dataset_id = d_id;

        -- Update dataset
        UPDATE datasets
        SET updated_at = CURRENT_TIMESTAMP
        WHERE id = d_id;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
