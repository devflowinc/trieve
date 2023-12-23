-- Your SQL goes here
CREATE FUNCTION update_tsvector() RETURNS TRIGGER AS $$
BEGIN
    NEW.card_metadata_tsvector := to_tsvector(NEW.content);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TRIGGER update_tsvector_trigger
BEFORE INSERT ON card_metadata
FOR EACH ROW
EXECUTE FUNCTION update_tsvector();
