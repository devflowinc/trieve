-- This file should undo anything in `up.sql`
-- Removing indices from the 'files' table
DROP INDEX IF EXISTS idx_files_user_id;
DROP INDEX IF EXISTS idx_files_private;
DROP INDEX IF EXISTS idx_files_created_at;
DROP INDEX IF EXISTS idx_files_updated_at;

-- Removing indices from the 'chunk_files' table
DROP INDEX IF EXISTS idx_chunk_files_chunk_id;
DROP INDEX IF EXISTS idx_chunk_files_file_id;
DROP INDEX IF EXISTS idx_chunk_files_created_at;
DROP INDEX IF EXISTS idx_chunk_files_updated_at;
