-- Your SQL goes here
ALTER TABLE
    users
ADD
    CONSTRAINT users_email_unique UNIQUE (email);

CREATE TABLE stripe_customers (
    id UUID NOT NULL UNIQUE PRIMARY KEY,
    stripe_id TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    FOREIGN KEY (email) REFERENCES users(email)
);