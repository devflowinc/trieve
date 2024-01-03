-- Your SQL goes here
CREATE OR REPLACE FUNCTION set_default_org_usage_data()
RETURNS TRIGGER AS $$
BEGIN
    -- Insert a new row if the organization doesn't exist
    INSERT INTO organization_usage_counts (org_id)
    VALUES (NEW.id)
    ON CONFLICT (org_id) DO NOTHING;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER set_default_org_usage_data_trigger
AFTER INSERT ON organizations
FOR EACH ROW
EXECUTE FUNCTION set_default_org_usage_data();

ALTER TABLE organizations DROP CONSTRAINT organizations_name_key;