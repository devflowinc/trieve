-- This file should undo anything in `up.sql`
ALTER TABLE stripe_invoices
DROP COLUMN stripe_id;
