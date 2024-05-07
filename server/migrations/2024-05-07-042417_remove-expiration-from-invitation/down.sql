-- This file should undo anything in `up.sql`
ALTER TABLE invitations ADD COLUMN expires_at timestamp;