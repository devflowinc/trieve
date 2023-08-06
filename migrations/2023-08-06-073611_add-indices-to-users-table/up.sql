-- Your SQL goes here
-- Index on id column (Primary Key is automatically indexed)
CREATE INDEX idx_users_id ON users (id);

-- Index on email column
CREATE INDEX idx_users_email ON users (email);

-- Index on hash column
CREATE INDEX idx_users_hash ON users (hash);

-- Index on username column
CREATE INDEX idx_users_username ON users (username);

-- Index on website column
CREATE INDEX idx_users_website ON users (website);

-- Index on created_at column
CREATE INDEX idx_users_created_at ON users (created_at);

-- Index on updated_at column
CREATE INDEX idx_users_updated_at ON users (updated_at);
