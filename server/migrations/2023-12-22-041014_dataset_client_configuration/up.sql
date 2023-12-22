-- Your SQL goes here
ALTER TABLE datasets
RENAME COLUMN configuration TO server_configuration;

ALTER TABLE datasets
ADD COLUMN client_configuration JSONB NOT NULL DEFAULT '{}';