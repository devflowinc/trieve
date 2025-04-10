-- This file should undo anything in `up.sql`
ALTER TABLE stripe_plans
RENAME COLUMN messages_per_month TO component_loads;