-- This file should undo anything in `up.sql`
ALTER TABLE stripe_usage_based_plans DROP COLUMN platform_price_id;
ALTER TABLE stripe_usage_based_plans DROP COLUMN platform_price_amount;
