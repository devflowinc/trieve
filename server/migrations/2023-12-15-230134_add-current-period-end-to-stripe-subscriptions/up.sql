-- Your SQL goes here
ALTER TABLE
    stripe_subscriptions
ADD
    COLUMN current_period_end TIMESTAMP DEFAULT NULL;