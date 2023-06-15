-- Your SQL goes here
CREATE TABLE card_collisions (
    id UUID NOT NULL,
    card_id UUID NOT NULL,
    collision_id UUID NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (card_id) REFERENCES card_metadata (id),
    FOREIGN KEY (collision_id) REFERENCES card_metadata (id)
)

CREATE INDEX idx_card_collisions_card_id ON card_collisions (card_id);
