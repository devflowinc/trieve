-- This file should undo anything in `up.sql`
ALTER TABLE files ADD COLUMN user_id UUID;
