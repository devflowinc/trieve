-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS update_dataset_counts_trigger ON dataset_usage_counts;
DROP TRIGGER IF EXISTS update_organization_counts_trigger ON organization_usage_counts;

DROP FUNCTION IF EXISTS update_dataset_counts();
DROP FUNCTION IF EXISTS update_organization_counts();

DROP TABLE IF EXISTS dataset_usage_counts;
DROP TABLE IF EXISTS organization_usage_counts;
CREATE TABLE IF NOT EXISTS chunk_metadata_counts (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    dataset_id UUID NOT NULL,
    total_rows BIGINT NOT NULL,
    FOREIGN KEY (dataset_id) REFERENCES datasets(id)
);

CREATE
OR REPLACE FUNCTION update_chunk_metadata_count() 
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO card_metadata_counts (id, dataset_id, total_rows)
        VALUES (null, NEW.dataset_id, 1)
        ON CONFLICT (dataset_id) DO UPDATE
        SET card_metadata_count = card_metadata_counts.total_rows + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE card_metadata_counts
        SET total_rows = card_metadata_counts.total_rows - 1
        WHERE dataset_id = OLD.dataset_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;