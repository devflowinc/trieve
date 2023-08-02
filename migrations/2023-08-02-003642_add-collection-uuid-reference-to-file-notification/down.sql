-- This file should undo anything in `up.sql`

-- Step 2: Recreate the original table without the foreign key constraint
CREATE TABLE file_upload_completed_notifications_temp (
    id UUID PRIMARY KEY,
    user_uuid UUID NOT NULL,
    collection_uuid UUID NOT NULL,
    user_read boolean NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Step 3: Copy data from the original table to the temporary table
INSERT INTO file_upload_completed_notifications_temp
SELECT * FROM file_upload_completed_notifications;

-- Step 4: Drop the original table with the foreign key constraint
DROP TABLE file_upload_completed_notifications;

-- Step 5: Recreate the original table without the foreign key constraint
CREATE TABLE file_upload_completed_notifications (
    id UUID PRIMARY KEY,
    user_uuid UUID NOT NULL,
    collection_uuid UUID NOT NULL,
    user_read boolean NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Step 6: Copy data back from the temporary table to the new original table
INSERT INTO file_upload_completed_notifications
SELECT * FROM file_upload_completed_notifications_temp;

-- Step 7: Drop the temporary table
DROP TABLE file_upload_completed_notifications_temp;
