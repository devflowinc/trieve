-- Your SQL goes here
ALTER TABLE organizations ADD CONSTRAINT organizations_name_key UNIQUE (name);
