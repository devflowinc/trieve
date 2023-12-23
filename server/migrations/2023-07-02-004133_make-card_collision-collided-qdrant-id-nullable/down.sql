-- This file should undo anything in `up.sql`
ALTER TABLE card_collisions ALTER COLUMN collision_qdrant_id DROP NOT NULL;
