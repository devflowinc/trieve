-- Your SQL goes here
-- make odic_subject nullable
ALTER TABLE users ALTER COLUMN oidc_subject DROP NOT NULL;
