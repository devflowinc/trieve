-- Your SQL goes here

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
    datasets_price_id TEXT NOT NULL,
    users_price_id TEXT NOT NULL,
    chunks_stored_price_id TEXT NOT NULL,
    files_storage_price_id TEXT NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE stripe_usage_based_subscriptions (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    stripe_subscription_id TEXT NOT NULL,
    usage_based_plan_id UUID NOT NULL REFERENCES stripe_usage_based_plans(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_recorded_meter TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Per Billing Cycle Metrics
    last_cycle_timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    last_cycle_dataset_count BIGINT NOT NULL DEFAULT 0,
    last_cycle_users_count INTEGER NOT NULL DEFAULT 0,
    last_cycle_chunks_stored_mb BIGINT NOT NULL DEFAULT 0,
    last_cycle_files_storage_mb BIGINT NOT NULL DEFAULT 0
);
