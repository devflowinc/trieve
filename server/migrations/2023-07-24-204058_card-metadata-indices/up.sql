-- Your SQL goes here
CREATE INDEX idx_card_metadata_private ON card_metadata (private);

CREATE INDEX idx_card_metadata_oc_file_path ON card_metadata (oc_file_path);

CREATE INDEX idx_card_metadata_link ON card_metadata (link);

CREATE INDEX idx_card_metadata_created_at ON card_metadata(created_at);

CREATE INDEX idx_card_metadata_updated_at ON card_metadata(updated_at);

CREATE INDEX idx_card_collisions_collision_qdrant_id ON card_collisions (collision_qdrant_id);

CREATE INDEX idx_card_metadata_id ON card_metadata (id);
