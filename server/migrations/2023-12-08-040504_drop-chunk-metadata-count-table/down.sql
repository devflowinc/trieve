-- This file should undo anything in `up.sql`
-- Create a new table to store row counts
CREATE TABLE chunk_metadata_count (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    total_rows BIGINT NOT NULL
);

-- Insert an initial count of rows (0 in this case)
INSERT INTO
    chunk_metadata_count (total_rows)
VALUES
    (
        (
            SELECT
                COUNT(*)
            FROM
                chunk_metadata
        )
    );

-- Create a trigger function to update the row count
CREATE
OR REPLACE FUNCTION update_chunk_metadata_count() RETURNS TRIGGER AS $ $ BEGIN IF TG_OP = 'INSERT' THEN
UPDATE
    chunk_metadata_count
SET
    total_rows = total_rows + 1;

ELSIF TG_OP = 'DELETE' THEN
UPDATE
    chunk_metadata_count
SET
    total_rows = total_rows - 1;

END IF;

RETURN NULL;

END;

$ $ LANGUAGE plpgsql;

-- Create a trigger to automatically update the count on INSERT or DELETE
CREATE TRIGGER chunk_metadata_count_trigger
AFTER
INSERT
    OR DELETE ON chunk_metadata FOR EACH ROW EXECUTE FUNCTION update_chunk_metadata_count();