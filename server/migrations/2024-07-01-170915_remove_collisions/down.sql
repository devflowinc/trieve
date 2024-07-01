-- This file should undo anything in `up.sql`
CREATE TABLE chunk_collisions (
	id uuid NOT NULL,
	chunk_id uuid NOT NULL,
	collision_qdrant_id uuid NULL,
	created_at timestamp DEFAULT now() NOT NULL,
	updated_at timestamp DEFAULT now() NOT NULL
);

ALTER TABLE chunk_metadata
ALTER COLUMN qdrant_point_id DROP NOT NULL;