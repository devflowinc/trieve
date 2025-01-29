-- Your SQL goes here
ALTER TABLE stripe_plans ADD COLUMN IF NOT EXISTS visible boolean NOT NULL DEFAULT false;
