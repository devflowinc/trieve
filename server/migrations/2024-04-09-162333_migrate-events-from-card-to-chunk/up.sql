-- Your SQL goes here
UPDATE events
SET event_type = REPLACE(event_type, 'card_', 'chunk_')
WHERE event_type LIKE 'card_%';