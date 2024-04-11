-- This file should undo anything in `up.sql`
ALTER TABLE datasets DROP INDEX datasets_organization_id_index;
ALTER TABLE stripe_subscriptions DROP INDEX stripe_subscriptions_plan_id_index;