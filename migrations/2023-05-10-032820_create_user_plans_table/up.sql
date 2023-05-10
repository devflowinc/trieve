-- Your SQL goes here
CREATE TABLE user_plans (
  id UUID NOT NULL UNIQUE PRIMARY KEY,
  user_id UUID NOT NULL,
  plan TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);
