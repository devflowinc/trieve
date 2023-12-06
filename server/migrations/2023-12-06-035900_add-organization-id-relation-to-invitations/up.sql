-- Your SQL goes here
ALTER TABLE
    invitations
ADD
    COLUMN organization_id UUID NOT NULL;

ALTER TABLE
    invitations
ADD
    CONSTRAINT invitations_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations(id);

ALTER TABLE
    invitations DROP COLUMN referral_tokens;