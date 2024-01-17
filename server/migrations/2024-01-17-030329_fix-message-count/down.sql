-- This file should undo anything in `up.sql`
CREATE OR REPLACE FUNCTION update_messages_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Update message_count for new messages or insert a new row if the organization doesn't exist
        INSERT INTO organization_usage_counts (org_id, message_count)
        VALUES ((SELECT organization_id FROM datasets WHERE id = OLD.dataset_id), 1)
        ON CONFLICT (org_id) DO UPDATE
        SET message_count = organization_usage_counts.message_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement message_count when a message is deleted
        UPDATE organization_usage_counts
        SET message_count = CASE WHEN organization_usage_counts.message_count > 0 THEN organization_usage_counts.message_count - 1 ELSE 0 END
        WHERE org_id = (SELECT organization_id FROM datasets WHERE id = OLD.dataset_id);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for messages
CREATE OR REPLACE TRIGGER update_messages_counts_trigger
AFTER INSERT OR DELETE ON messages
FOR EACH ROW
EXECUTE FUNCTION update_messages_counts();