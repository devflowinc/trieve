-- Your SQL goes here
CREATE INDEX idx_events_dataset_id ON events (dataset_id);
CREATE INDEX idx_events_event_type ON events (event_type);