-- Your SQL goes here
ALTER TABLE
    datasets
ADD
    COLUMN organization_id UUID DEFAULT NULL;

UPDATE
    datasets
SET
    organization_id = organizations.id
FROM
    organizations
WHERE
    organizations.name = 'DEFAULT';

ALTER TABLE
    datasets
ALTER COLUMN
    organization_id
SET
    NOT NULL;

ALTER TABLE
    datasets
ADD
    CONSTRAINT datasets_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations(id);