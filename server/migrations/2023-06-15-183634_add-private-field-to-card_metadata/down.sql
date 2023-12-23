-- Remove the CHECK constraint
ALTER TABLE card_metadata
DROP CONSTRAINT qdrant_point_nullable_constraint;

-- Alter the qdrant_point_id column to disallow NULL values
ALTER TABLE card_metadata
ALTER COLUMN qdrant_point_id SET NOT NULL;

-- Remove the private field
ALTER TABLE card_metadata
DROP COLUMN private;
