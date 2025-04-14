-- This file should undo anything in `up.sql`
delete from stripe_plans where id = 'dead0000-f0ee-4000-a000-000000000000';
