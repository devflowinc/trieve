-- Your SQL goes here
ALTER TABLE chunk_collection RENAME TO chunk_group;
ALTER TABLE chunk_collection_bookmarks RENAME TO chunk_group_bookmarks;

ALTER TABLE collections_from_files RENAME TO groups_from_files;
ALTER TABLE file_upload_completed_notifications RENAME COLUMN collection_uuid TO group_uuid;

ALTER TABLE groups_from_files RENAME COLUMN collection_id TO group_id;
ALTER TABLE chunk_group_bookmarks RENAME COLUMN collection_id TO group_id;

ALTER TABLE dataset_collection_counts RENAME TO dataset_group_counts;
ALTER TABLE dataset_group_counts RENAME COLUMN collection_count TO group_count;
ALTER TABLE dataset_group_counts ADD CONSTRAINT UQ_dataset_id UNIQUE (dataset_id);

CREATE OR REPLACE FUNCTION public.update_collection_counts()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
        INSERT INTO dataset_group_counts (id, dataset_id, group_count)
        VALUES (NEW.id, NEW.dataset_id, 1)
        ON CONFLICT (dataset_id) DO UPDATE
        SET group_count = dataset_group_counts.group_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE dataset_group_counts
        SET group_count = dataset_group_counts.group_count - 1
        WHERE dataset_id = OLD.dataset_id;
    END IF;
    RETURN NULL;
END;
$function$
;