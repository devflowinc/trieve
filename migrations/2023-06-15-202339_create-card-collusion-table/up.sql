-- Your SQL goes here
-- Your SQL goes here
CREATE TABLE card_collisions (
    id UUID PRIMARY KEY,
    card_id UUID NOT NULL REFERENCES card_metadata (id),
    collision_id UUID NOT NULL REFERENCES card_metadata (id)
);

CREATE INDEX idx_card_collisions_card_id ON card_collisions (card_id);
