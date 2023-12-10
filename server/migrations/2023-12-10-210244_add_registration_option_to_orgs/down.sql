-- This file should undo anything in `up.sql`
ALTER TABLE orgs DROP COLUMN IF EXISTS registerable;
