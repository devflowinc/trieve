CREATE INDEX idx_link_gin ON chunk_metadata USING gin (link gin_trgm_ops);

CREATE INDEX json_gin ON chunk_metadata USING gin (metadata);

CREATE INDEX idx_gist ON chunk_metadata USING gin (tag_set gin_trgm_ops);

CREATE INDEX idx_card_metadata_updated_at ON chunk_metadata(updated_at);

CREATE INDEX idx_card_metadata_created_at ON chunk_metadata(created_at);

CREATE INDEX card_time_stamp_index ON chunk_metadata(time_stamp);
