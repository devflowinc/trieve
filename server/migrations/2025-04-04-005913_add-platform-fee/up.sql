-- Your SQL goes here
ALTER TABLE stripe_usage_based_plans ADD COLUMN platform_price_id TEXT NULL;
ALTER TABLE stripe_usage_based_plans ADD COLUMN platform_price_amount INTEGER NULL;
