-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS user_plans;
DROP INDEX IF EXISTS idx_user_plans_stripe_customer_id;
