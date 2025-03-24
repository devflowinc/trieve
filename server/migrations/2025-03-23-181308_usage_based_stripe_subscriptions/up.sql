-- Your SQL goes here
CREATE TABLE usage_based_stripe_subscriptions (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    stripe_subscription_id TEXT NOT NULL,
    last_recorded_meter TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

create TABLE stripe_usage_based_plans (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    visible BOOLEAN NOT NULL,
    ingest_tokens_price_id TEXT NOT NULL,
    bytes_ingested_price_id TEXT NOT NULL,
    search_tokens_price_id TEXT NOT NULL,
    message_tokens_price_id TEXT NOT NULL,
    analytics_events_price_id TEXT NOT NULL,
    ocr_pages_price_id TEXT NOT NULL,
    pages_crawls_price_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
