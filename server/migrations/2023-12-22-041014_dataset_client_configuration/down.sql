-- This file should undo anything in `up.sql`
DROP COLUMN IF EXISTS client_configuration;

ALTER TABLE datasets
RENAME COLUMN IF EXISTS server_configuration TO configuration;