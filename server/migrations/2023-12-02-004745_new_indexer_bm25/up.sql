-- Your SQL goes here
DROP INDEX idx_content_search IF EXISTS;

CREATE INDEX idx_content_search
ON card_metadata
USING bm25 ((card_metadata.*))
WITH (text_fields='{"card_html": {tokenizer: {type: "ngram", min_gram: 2, max_gram: 10}}}}');