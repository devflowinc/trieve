-- Drop the new trigger that handles 'INSERT', 'UPDATE', and 'DELETE'
DROP TRIGGER IF EXISTS update_files_storage_with_update_trigger ON files;

-- Drop the modified function that includes the 'UPDATE' logic
DROP FUNCTION IF EXISTS update_files_storage_counts_with_update();
