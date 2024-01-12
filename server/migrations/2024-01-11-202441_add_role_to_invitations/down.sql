-- This file should undo anything in `up.sql`
ALTER TABLE invitations DROP COLUMN IF EXISTS role;
