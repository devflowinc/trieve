-- This file should undo anything in `up.sql`
ALTER TABLE stripe_plans DROP COLUMN IF EXISTS visible;
