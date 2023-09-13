-- Your SQL goes here
-- Create a new table to store row counts
-- Create a new table to store row counts
CREATE TABLE card_metadata_count (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    total_rows BIGINT NOT NULL
);

-- Insert an initial count of rows (0 in this case)
INSERT INTO card_metadata_count (total_rows)
VALUES ((SELECT COUNT(*) FROM card_metadata));

-- Create a trigger function to update the row count
CREATE OR REPLACE FUNCTION update_card_metadata_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE card_metadata_count
        SET total_rows = total_rows + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE card_metadata_count
        SET total_rows = total_rows - 1;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create a trigger to automatically update the count on INSERT or DELETE
CREATE TRIGGER card_metadata_count_trigger
AFTER INSERT OR DELETE
ON card_metadata
FOR EACH ROW
EXECUTE FUNCTION update_card_metadata_count();

