-- This file should undo anything in `up.sql`
ALTER TABLE
    invitations DROP COLUMN IF EXISTS organization_id;

ALTER TABLE
    invitations
ADD
    COLUMN referral_tokens TEXT;