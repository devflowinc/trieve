-- This file should undo anything in `up.sql`
DROP TRIGGER card_metadata_count_trigger ON card_metadata;

DROP FUNCTION update_card_metadata_count();

DROP TABLE card_metadata_counts;