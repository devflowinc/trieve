-- Your SQL goes here
create or replace trigger update_event_count after
insert
    or
delete
    or
update
    on
    public.events for each row execute function update_notification_count();