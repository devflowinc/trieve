-- This file should undo anything in `up.sql`
ALTER TABLE
    users DROP CONSTRAINT users_email_unique;

DROP TABLE IF EXISTS stripe_customers;