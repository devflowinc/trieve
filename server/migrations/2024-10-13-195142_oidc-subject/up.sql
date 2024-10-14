-- Your SQL goes here
ALTER TABLE users ADD COLUMN oidc_subject VARCHAR(255) UNIQUE;
UPDATE users SET oidc_subject = id;
ALTER TABLE users ALTER COLUMN oidc_subject SET NOT NULL;
