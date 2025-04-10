-- Your SQL goes here
ALTER TABLE stripe_plans
RENAME COLUMN component_loads TO messages_per_month;
