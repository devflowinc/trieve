-- Your SQL goes here
CREATE TABLE user_plans (
  id UUID NOT NULL UNIQUE PRIMARY KEY,
  stripe_customer_id TEXT NOT NULL REFERENCES stripe_customers(stripe_id) MATCH SIMPLE,
  plan TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);
