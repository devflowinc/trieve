-- This file should undo anything in `up.sql`
ALTER TABLE chunk_collisions ALTER COLUMN collision_qdrant_id DROP NOT NULL;
