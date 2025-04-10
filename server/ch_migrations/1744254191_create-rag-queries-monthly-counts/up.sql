CREATE MATERIALIZED VIEW mv_rag_queries_monthly_counts
ENGINE = SummingMergeTree()
ORDER BY (dataset_id, month_year)
POPULATE AS
SELECT 
    dataset_id,
    formatDateTime(created_at, '%Y-%m') AS month_year,
    COUNT(*) AS query_count
FROM rag_queries
GROUP BY dataset_id, month_year;
