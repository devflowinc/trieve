-- Your SQL goes here
CREATE TABLE chunk_boosts (
  chunk_id UUID NOT NULL,
  fulltext_boost_phrase TEXT,
  fulltext_boost_factor FLOAT,
  semantic_boost_phrase TEXT,
  semantic_boost_factor FLOAT,

  PRIMARY KEY (chunk_id),
  CONSTRAINT chunk_boosts_chunk_id_fkey FOREIGN KEY (chunk_id) REFERENCES chunk_metadata (id) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT fulltext_pairs CHECK ((fulltext_boost_phrase IS NULL AND fulltext_boost_factor IS NULL) OR 
                                 (fulltext_boost_phrase IS NOT NULL AND fulltext_boost_factor IS NOT NULL)),
  CONSTRAINT semantic_pairs CHECK ((semantic_boost_phrase IS NULL AND semantic_boost_factor IS NULL) OR 
                                 (semantic_boost_phrase IS NOT NULL AND semantic_boost_factor IS NOT NULL))
)
