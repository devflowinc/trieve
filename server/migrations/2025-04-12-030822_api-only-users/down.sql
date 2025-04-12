-- This file should undo anything in `up.sql`
ALTER TABLE users
ALTER COLUMN oidc_subject SET NOT NULL;
