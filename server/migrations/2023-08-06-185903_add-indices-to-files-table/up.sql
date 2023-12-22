-- Your SQL goes here
-- Adding indices to the 'files' table
CREATE INDEX idx_files_user_id ON files (user_id);
CREATE INDEX idx_files_private ON files (private);
CREATE INDEX idx_files_created_at ON files (created_at);
CREATE INDEX idx_files_updated_at ON files (updated_at);

-- Adding indices to the 'chunk_files' table
CREATE INDEX idx_chunk_files_chunk_id ON chunk_files (chunk_id);
CREATE INDEX idx_chunk_files_file_id ON chunk_files (file_id);
CREATE INDEX idx_chunk_files_created_at ON chunk_files (created_at);
CREATE INDEX idx_chunk_files_updated_at ON chunk_files (updated_at);
