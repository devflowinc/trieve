-- Your SQL goes here
ALTER TABLE user_api_key ADD COLUMN dataset_ids TEXT[] DEFAULT NULL;
ALTER TABLE user_api_key ADD COLUMN organization_ids TEXT[] DEFAULT NULL;