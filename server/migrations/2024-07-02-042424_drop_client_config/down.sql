-- This file should undo anything in `up.sql`
ALTER TABLE datasets ADD COLUMN client_configuration jsonb;