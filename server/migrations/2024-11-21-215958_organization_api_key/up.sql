-- Your SQL goes here
CREATE TABLE organization_api_key (
	id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
	organization_id uuid NOT NULL,
	api_key_hash text NOT NULL UNIQUE,
	name text NOT NULL DEFAULT 'default'::text,
	created_at timestamp NOT NULL DEFAULT now(),
	updated_at timestamp NOT NULL DEFAULT now(),
	"role" int4 NOT NULL DEFAULT 0,
	dataset_ids _text NULL,
	scopes _text NULL,
    params jsonb NULL,
	expires_at timestamp NULL,
    FOREIGN KEY (organization_id) REFERENCES organizations(id)
);

CREATE INDEX organization_api_key_api_key_hash_index ON organization_api_key USING btree (api_key_hash);
