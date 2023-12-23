-- This file should undo anything in `up.sql`
ALTER TABLE
    card_metadata
ADD
    COLUMN private boolean NOT NULL DEFAULT false;

ALTER TABLE
    files
ADD
    COLUMN private boolean NOT NULL DEFAULT false;
