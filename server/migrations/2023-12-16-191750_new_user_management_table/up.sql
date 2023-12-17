-- Your SQL goes here
ALTER TABLE users DROP COLUMN IF EXISTS organization_id;

CREATE TABLE user_organizations (
  id UUID NOT NULL UNIQUE PRIMARY KEY,
  user_id UUID NOT NULL,
  organization_id UUID NOT NULL,
  role integer NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

ALTER TABLE user_organizations ADD CONSTRAINT fk_user_id FOREIGN KEY (user_id) REFERENCES users (id);
ALTER TABLE user_organizations ADD CONSTRAINT fk_organization_id FOREIGN KEY (organization_id) REFERENCES organizations (id);
