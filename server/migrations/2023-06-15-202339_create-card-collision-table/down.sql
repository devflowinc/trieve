-- This file should undo anything in `up.sql`
DROP TABLE card_collisions;
ALTER TABLE card_metadata
DROP CONSTRAINT uq_qdrant_point_id;
