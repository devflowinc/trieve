-- Your SQL goes here
CREATE TABLE stripe_plans (
    id UUID NOT NULL UNIQUE PRIMARY KEY,
    stripe_id TEXT NOT NULL UNIQUE,
    card_count INTEGER NOT NULL DEFAULT 0,
    file_storage INTEGER NOT NULL DEFAULT 0,
    user_count INTEGER NOT NULL DEFAULT 0,
    dataset_count INTEGER NOT NULL DEFAULT 0,
    message_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE stripe_subscriptions (
    id UUID NOT NULL UNIQUE PRIMARY KEY,
    stripe_id TEXT NOT NULL UNIQUE,
    stripe_plan_id TEXT NOT NULL,
    stripe_customer_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    FOREIGN KEY (stripe_plan_id) REFERENCES stripe_plans(stripe_id),
    FOREIGN KEY (stripe_customer_id) REFERENCES stripe_customers(stripe_id)
);