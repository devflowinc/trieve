CREATE MATERIALIZED VIEW mv_rag_queries_monthly_counts
ENGINE = SummingMergeTree()
ORDER BY (organization_id, month_year)
POPULATE AS
SELECT 
    organization_id,
    formatDateTime(created_at, '%Y-%m') AS month_year,
    COUNT(*) AS query_count
FROM rag_queries
GROUP BY organization_id, month_year;
