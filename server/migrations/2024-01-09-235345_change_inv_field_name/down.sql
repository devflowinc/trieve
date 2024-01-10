-- This file should undo anything in `up.sql`
ALTER TABLE invitations RENAME COLUMN organization_id TO dataset_id;
