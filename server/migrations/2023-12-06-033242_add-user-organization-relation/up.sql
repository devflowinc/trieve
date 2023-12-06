-- Your SQL goes here
ALTER TABLE
    users
ADD
    COLUMN organization_id UUID NOT NULL;

ALTER TABLE
    users
ADD
    CONSTRAINT users_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations(id);