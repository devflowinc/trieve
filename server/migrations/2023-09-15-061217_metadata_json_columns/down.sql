-- Revert the changes made to the 'card_metadata' table
-- Restore the original column names and drop the 'metadata' column


-- Rename 'filter_two' back to 'oc_file_path'
ALTER TABLE card_metadata
RENAME COLUMN tag_set TO oc_file_path;

-- Drop the 'metadata' column
ALTER TABLE card_metadata
DROP COLUMN metadata;

-- Revert the changes made to the 'files' table
-- Rename 'filter_two' back to 'oc_file_path'

ALTER TABLE files
RENAME COLUMN tag_set TO oc_file_path;
