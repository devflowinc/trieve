-- This file should undo anything in `up.sql`
ALTER TABLE topics ADD COLUMN side VARCHAR(255);
ALTER TABLE topics ADD COLUMN normal_chat BOOLEAN DEFAULT TRUE;