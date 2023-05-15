-- This file should undo anything in `up.sql`
-- Step 1: Create a temporary table with the desired schema
ALTER TABLE users
ALTER COLUMN email TYPE VARCHAR(100),
ALTER COLUMN hash TYPE VARCHAR(122);
