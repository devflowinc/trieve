-- Your SQL goes here
CREATE TABLE file_upload_completed_notifications (
    id UUID PRIMARY KEY,
    user_uuid UUID NOT NULL,
    collection_uuid UUID NOT NULL,
    user_read boolean NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
); 
