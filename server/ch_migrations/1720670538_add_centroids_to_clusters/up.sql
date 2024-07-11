ALTER TABLE cluster_topics
ADD COLUMN IF NOT EXISTS centroid Array(Float32) DEFAULT [];