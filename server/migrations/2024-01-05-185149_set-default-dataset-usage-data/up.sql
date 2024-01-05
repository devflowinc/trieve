-- Your SQL goes here
CREATE OR REPLACE FUNCTION set_default_dataset_usage_data()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO dataset_usage_counts (dataset_id)
    VALUES (NEW.id)
    ON CONFLICT (dataset_id) DO NOTHING;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER set_default_dataset_usage_data_trigger
AFTER INSERT ON datasets
FOR EACH ROW
EXECUTE FUNCTION set_default_dataset_usage_data();