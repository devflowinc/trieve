-- Your SQL goes here
ALTER TABLE chunk_collisions
DROP CONSTRAINT card_collisions_card_id_fkey;

ALTER TABLE chunk_collisions
ADD CONSTRAINT chunk_collisions_card_id_fkey FOREIGN KEY (chunk_id) REFERENCES chunk_metadata(id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE chunk_collisions
DROP CONSTRAINT card_collisions_collision_qdrant_id_fkey;

ALTER TABLE chunk_collisions
ADD CONSTRAINT chunk_collisions_collision_qdrant_id_fkey FOREIGN KEY (collision_qdrant_id) REFERENCES chunk_metadata(qdrant_point_id) ON DELETE CASCADE ON UPDATE CASCADE;

