-- Your SQL goes here
ALTER TABLE chunk_collection DROP COLUMN author_id;
ALTER TABLE user_collection_counts DROP COLUMN user_id;
ALTER TABLE user_collection_counts ADD COLUMN dataset_id uuid;
ALTER TABLE user_collection_counts RENAME TO dataset_collection_counts;

CREATE OR REPLACE FUNCTION public.update_collection_counts()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
        INSERT INTO dataset_collection_counts (id, user_id, collection_count)
        VALUES (NEW.id, NEW.author_id, 1)
        ON CONFLICT (user_id) DO UPDATE
        SET collection_count = dataset_collection_counts.collection_count + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE dataset_collection_counts
        SET collection_count = dataset_collection_counts.collection_count - 1
        WHERE user_id = OLD.author_id;
    END IF;
    RETURN NULL;
END;
$function$
;
