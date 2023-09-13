-- This file should undo anything in `up.sql`
ALTER TABLE invitations
DROP COLUMN referral_tokens;
