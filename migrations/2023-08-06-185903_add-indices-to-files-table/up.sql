-- Your SQL goes here
-- Adding indices to the 'files' table
CREATE INDEX idx_files_user_id ON files (user_id);
CREATE INDEX idx_files_private ON files (private);
CREATE INDEX idx_files_created_at ON files (created_at);
CREATE INDEX idx_files_updated_at ON files (updated_at);

-- Adding indices to the 'card_files' table
CREATE INDEX idx_card_files_card_id ON card_files (card_id);
CREATE INDEX idx_card_files_file_id ON card_files (file_id);
CREATE INDEX idx_card_files_created_at ON card_files (created_at);
CREATE INDEX idx_card_files_updated_at ON card_files (updated_at);
