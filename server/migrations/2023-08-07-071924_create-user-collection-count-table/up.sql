-- Your SQL goes here
CREATE TABLE user_collection_counts (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    user_id UUID UNIQUE NOT NULL REFERENCES users(id),
    collection_count INTEGER NOT NULL DEFAULT 0
);

CREATE OR REPLACE FUNCTION update_collection_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
        INSERT INTO user_collection_counts (id, user_id, collection_count)
        VALUES (NEW.id, NEW.author_id, 1)
        ON CONFLICT (user_id) DO UPDATE
        SET collection_count = user_collection_counts.collection_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE user_collection_counts
        SET collection_count = user_collection_counts.collection_count - 1
        WHERE user_id = OLD.author_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_collection_counts_trigger
AFTER INSERT OR UPDATE OR DELETE ON card_collection
FOR EACH ROW
EXECUTE FUNCTION update_collection_counts();

-- Initialize user_collection_counts with existing data
INSERT INTO user_collection_counts (id, user_id, collection_count)
SELECT DISTINCT ON (author_id) gen_random_uuid(), author_id, (SELECT COUNT(*) FROM card_collection WHERE author_id = author_id)
FROM card_collection;