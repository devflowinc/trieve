-- This file should undo anything in `up.sql`
ALTER TABLE
    stripe_subscriptions DROP COLUMN current_period_end;