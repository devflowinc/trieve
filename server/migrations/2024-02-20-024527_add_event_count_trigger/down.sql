-- This file should undo anything in `up.sql`
drop trigger if exists update_event_count on public.events;
