-- Your SQL goes here
CREATE TABLE chunk_metadata_counts (
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
        INSERT INTO chunk_metadata_counts (id, dataset_id, total_rows)
        VALUES (null, NEW.dataset_id, 1)
        ON CONFLICT (dataset_id) DO UPDATE
        SET chunk_metadata_count = chunk_metadata_counts.total_rows + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE chunk_metadata_counts
        SET total_rows = chunk_metadata_counts.total_rows - 1
        WHERE dataset_id = OLD.dataset_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create a trigger to automatically update the count on INSERT or DELETE
CREATE TRIGGER chunk_metadata_count_trigger
AFTER
INSERT
    OR DELETE ON chunk_metadata FOR EACH ROW EXECUTE FUNCTION update_chunk_metadata_count();