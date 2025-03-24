-- Your SQL goes here
CREATE TABLE usage_based_stripe_subscriptions (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    stripe_subscription_id TEXT NOT NULL,
    last_recorded_meter TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
