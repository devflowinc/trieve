-- Your SQL goes here
CREATE INDEX IF NOT EXISTS datasets_organization_id_index ON datasets (organization_id);
CREATE INDEX IF NOT EXISTS stripe_subscriptions_plan_id_index ON stripe_subscriptions (plan_id);