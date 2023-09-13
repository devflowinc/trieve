-- This file should undo anything in `up.sql`
DROP INDEX idx_card_metadata_private;

DROP INDEX idx_card_metadata_oc_file_path;

DROP INDEX idx_card_metadata_link;

DROP INDEX idx_card_metadata_created_at;

DROP INDEX idx_card_metadata_updated_at;

DROP INDEX idx_card_collisions_collision_qdrant_id;

DROP INDEX idx_card_metadata_id;
