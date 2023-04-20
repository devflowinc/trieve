-- Your SQL goes here
CREATE TABLE users (
  id UUID NOT NULL UNIQUE PRIMARY KEY,
  email VARCHAR(100) NOT NULL UNIQUE,
  hash VARCHAR(122) NOT NULL, --argon hash
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);
