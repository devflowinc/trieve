-- This file should undo anything in `up.sql`
-- Your SQL goes here
ALTER TABLE chunk_verification DROP COLUMN created_at;
ALTER TABLE chunk_verification DROP COLUMN updated_at;

DROP FUNCTION IF EXISTS update_updated_at() CASCADE;
DROP FUNCTION IF EXISTS update_main_table_updated_at() CASCADE;

DROP TRIGGER IF EXISTS update_updated_at ON chunk_collection;
DROP TRIGGER IF EXISTS update_updated_at ON chunk_collection_bookmarks;
DROP TRIGGER IF EXISTS update_updated_at ON chunk_collisions;
DROP TRIGGER IF EXISTS update_updated_at ON chunk_files;
DROP TRIGGER IF EXISTS update_updated_at ON chunk_metadata;
DROP TRIGGER IF EXISTS update_updated_at ON chunk_verification;
DROP TRIGGER IF EXISTS update_updated_at ON chunk_votes;
DROP TRIGGER IF EXISTS update_updated_at ON collections_from_files;
DROP TRIGGER IF EXISTS update_updated_at ON files;
DROP TRIGGER IF EXISTS update_updated_at ON invitations;
DROP TRIGGER IF EXISTS update_updated_at ON messages;
DROP TRIGGER IF EXISTS update_updated_at ON password_resets;
DROP TRIGGER IF EXISTS update_updated_at ON stripe_customers;
DROP TRIGGER IF EXISTS update_updated_at ON topics;
DROP TRIGGER IF EXISTS update_updated_at ON user_plans;
DROP TRIGGER IF EXISTS update_updated_at ON users;
DROP TRIGGER IF EXISTS update_updated_at ON verification_notifications;