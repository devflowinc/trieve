-- This file should undo anything in `up.sql`
UPDATE events
SET event_type = REPLACE(event_type, 'chunk_', 'card_')
WHERE event_type LIKE 'chunk_%';
