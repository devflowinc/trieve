-- Your SQL goes here
-- Your SQL goes here
ALTER TABLE card_metadata
ADD CONSTRAINT uq_qdrant_point_id UNIQUE (qdrant_point_id);

CREATE TABLE card_collisions (
    id UUID PRIMARY KEY,
    card_id UUID NOT NULL REFERENCES card_metadata (id),
    collision_qdrant_id UUID NOT NULL REFERENCES card_metadata (qdrant_point_id)
);

CREATE INDEX idx_card_collisions_card_id ON card_collisions (card_id);
