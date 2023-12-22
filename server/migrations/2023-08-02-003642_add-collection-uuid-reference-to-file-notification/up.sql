-- Your SQL goes here
-- Step 1: Create a new table without foreign key constraint
CREATE TABLE file_upload_completed_notifications_temp (
    id UUID PRIMARY KEY,
    user_uuid UUID NOT NULL,
    collection_uuid UUID NOT NULL,
    user_read boolean NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Step 2: Copy data from the original table to the new table
INSERT INTO file_upload_completed_notifications_temp
SELECT * FROM file_upload_completed_notifications;

-- Step 3: Drop the original table
DROP TABLE file_upload_completed_notifications;

-- Step 4: Recreate the original table with the foreign key constraint
CREATE TABLE file_upload_completed_notifications (
    id UUID PRIMARY KEY,
    user_uuid UUID NOT NULL,
    collection_uuid UUID NOT NULL REFERENCES chunk_collection(id),
    user_read boolean NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Step 5: Copy data back from the temporary table to the original table
INSERT INTO file_upload_completed_notifications
SELECT * FROM file_upload_completed_notifications_temp;

-- Step 6: Drop the temporary table
DROP TABLE file_upload_completed_notifications_temp;
