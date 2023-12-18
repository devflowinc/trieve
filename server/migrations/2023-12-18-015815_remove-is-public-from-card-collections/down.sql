-- This file should undo anything in `up.sql`
ALTER TABLE
    card_collection
ADD
    COLUMN is_public boolean NOT NULL DEFAULT true;