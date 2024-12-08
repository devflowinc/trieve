ALTER TABLE rag_queries ADD COLUMN IF NOT EXISTS hallucination_score Float64 DEFAULT 0.0;
ALTER TABLE rag_queries ADD COLUMN IF NOT EXISTS detected_hallucinations Array(String) DEFAULT [];

