-- Your SQL goes here
CREATE TABLE stripe_plans (
    id UUID NOT NULL UNIQUE PRIMARY KEY,
    stripe_id TEXT NOT NULL UNIQUE,
    card_count INTEGER NOT NULL DEFAULT 0,
    file_storage INTEGER NOT NULL DEFAULT 0,
    user_count INTEGER NOT NULL DEFAULT 0,
    dataset_count INTEGER NOT NULL DEFAULT 0,
    message_count INTEGER NOT NULL DEFAULT 0,
    amount BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE stripe_subscriptions (
    id UUID NOT NULL UNIQUE PRIMARY KEY,
    stripe_id TEXT NOT NULL UNIQUE,
    plan_id UUID NOT NULL REFERENCES stripe_plans(id),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
