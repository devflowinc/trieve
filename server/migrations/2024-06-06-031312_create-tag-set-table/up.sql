-- Your SQL goes here
CREATE TABLE dataset_tags (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id UUID NOT NULL,
    tag TEXT NOT NULL UNIQUE,

    CONSTRAINT dataset_tags_id_dataset_id UNIQUE(id, dataset_id),
    FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE chunk_metadata_tags (
    id UUID NOT NULL UNIQUE PRIMARY KEY DEFAULT gen_random_uuid(),

    chunk_metadata_id UUID NOT NULL,
    tag_id UUID NOT NULL,

    CONSTRAINT chunk_metadata_tags_metadata_id_tag_id UNIQUE(chunk_metadata_id, tag_id),

    FOREIGN KEY (chunk_metadata_id) REFERENCES chunk_metadata(id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES dataset_tags(id) ON UPDATE CASCADE ON DELETE CASCADE
)
