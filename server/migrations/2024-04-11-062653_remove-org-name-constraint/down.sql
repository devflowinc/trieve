-- This file should undo anything in `up.sql`
ALTER TABLE organizations ADD CONSTRAINT organizations_name_key UNIQUE (name);