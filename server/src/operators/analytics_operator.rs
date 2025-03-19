use crate::{
    data::models::{
        CTRMetricsOverTimePoint, CTRMetricsOverTimeResponse, ClusterAnalyticsFilter,
        ClusterTopicsClickhouse, ComponentAnalyticsFilter, ComponentNamesResponse,
        DatasetAnalytics, EventAnalyticsFilter, EventData, EventDataClickhouse,
        GetEventsResponseBody, Granularity, HeadQueries, MessagesPerUserResponse,
        MessagesPerUserTimePointClickhouse, Pool, PopularFilters, PopularFiltersClickhouse,
        RAGAnalyticsFilter, RAGSortBy, RAGUsageGraphResponse, RAGUsageResponse, RagQueryEvent,
        RagQueryEventClickhouse, RagQueryRatingsResponse, RecommendationAnalyticsFilter,
        RecommendationCTRMetrics, RecommendationEvent, RecommendationEventClickhouse,
        RecommendationUsageGraphPoint, RecommendationUsageGraphPointClickhouse,
        RecommendationUsageGraphResponse, RecommendationsCTRRateResponse,
        RecommendationsCTRRateTimePoint, RecommendationsPerUserResponse,
        RecommendationsPerUserTimePoint, RecommendationsPerUserTimePointClickhouse,
        RecommendationsWithClicksCTRResponse, RecommendationsWithClicksCTRResponseClickhouse,
        RecommendationsWithoutClicksCTRResponse, RecommendationsWithoutClicksCTRResponseClickhouse,
        SearchAnalyticsFilter, SearchCTRMetrics, SearchCTRMetricsClickhouse, SearchClusterTopics,
        SearchLatencyGraph, SearchLatencyGraphClickhouse, SearchQueriesWithClicksCTRResponse,
        SearchQueriesWithClicksCTRResponseClickhouse, SearchQueriesWithoutClicksCTRResponse,
        SearchQueriesWithoutClicksCTRResponseClickhouse, SearchQueryEvent,
        SearchQueryEventClickhouse, SearchSortBy, SearchTypeCount, SortOrder, TopComponents,
        TopComponentsResponse, TopDatasetsResponse, TopDatasetsResponseClickhouse, TopPages,
        TopPagesResponse, TopicAnalyticsFilter, TopicAnalyticsSummaryClickhouse,
        TopicDetailsResponse, TopicQueriesResponse, TopicQueryClickhouse, TopicTimePointClickhouse,
        TopicsOverTimeResponse, TotalUniqueUsersResponse, TotalUniqueUsersTimePointClickhouse,
        UsageGraphPoint, UsageGraphPointClickhouse,
    },
    errors::ServiceError,
    handlers::analytics_handler::GetTopDatasetsRequestBody,
};
use actix_web::web;

use clickhouse::Row;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::future::join_all;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "SearchClusterResponse")]
pub struct SearchClusterResponse {
    pub clusters: Vec<SearchClusterTopics>,
}

pub async fn get_clusters_query(
    dataset_id: uuid::Uuid,
    filters: Option<ClusterAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchClusterResponse, ServiceError> {
    let mut query_string = String::from("SELECT ?fields FROM cluster_topics WHERE dataset_id = ?");

    if let Some(filters) = filters {
        query_string = filters.add_to_query(query_string);
    }

    query_string.push_str(" ORDER BY density DESC LIMIT 10");

    let clickhouse_topics = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<ClusterTopicsClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching topics: {:?}", e);
            ServiceError::InternalServerError("Error fetching topics".to_string())
        })?;

    let topics: Vec<SearchClusterTopics> = clickhouse_topics
        .into_iter()
        .map(|t| t.into())
        .collect::<Vec<_>>();

    Ok(SearchClusterResponse { clusters: topics })
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "SearchQueryResponse")]
pub struct SearchQueryResponse {
    pub queries: Vec<SearchQueryEvent>,
}

pub async fn get_queries_for_cluster_query(
    dataset_id: uuid::Uuid,
    cluster_id: uuid::Uuid,
    page: Option<u32>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchQueryResponse, ServiceError> {
    let query_string = String::from(
        "
        SELECT DISTINCT ON (search_queries.request_params, search_queries.query) ?fields 
        FROM search_queries 
        JOIN search_cluster_memberships ON search_queries.id = search_cluster_memberships.search_id 
        WHERE search_cluster_memberships.cluster_id = ? 
            AND search_queries.dataset_id = ? AND search_queries.is_duplicate = 0
        ORDER BY
            search_cluster_memberships.distance_to_centroid DESC
        LIMIT 15 
        OFFSET ?
    ",
    );

    let clickhouse_queries = clickhouse_client
        .query(query_string.as_str())
        .bind(cluster_id)
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 15)
        .fetch_all::<SearchQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching queries: {:?}", e);
            ServiceError::InternalServerError("Error fetching queries".to_string())
        })?;

    let queries: Vec<SearchQueryEvent> = clickhouse_queries
        .into_iter()
        .map(|q| q.into())
        .collect_vec();

    Ok(SearchQueryResponse { queries })
}

pub async fn get_search_query(
    dataset_id: uuid::Uuid,
    search_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchQueryEvent, ServiceError> {
    let mut clickhouse_query = clickhouse_client
        .query("SELECT ?fields FROM search_queries WHERE id = ? AND dataset_id = ?")
        .bind(search_id)
        .bind(dataset_id)
        .fetch_one::<SearchQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let re = regex::Regex::new(r#""chunk_html":\s*"(.*?)"\s*,\s*"metadata""#).unwrap();
    let results = clickhouse_query.results.clone();

    let mut chunk_htmls = vec!["".to_string(); results.len()];

    for (i, result_chunk) in results.iter().enumerate() {
        if let Some(captures) = re.captures(result_chunk) {
            if let Some(content) = captures.get(1) {
                let result = re.replace(result_chunk.as_str(), |_: &regex::Captures| {
                    r#""chunk_html": "","metadata""#.to_string()
                });
                chunk_htmls[i] = content.as_str().to_string();
                clickhouse_query.results[i] = result.to_string();
            }
        }
    }

    let mut query: SearchQueryEvent = clickhouse_query.into();

    query.results = query
        .results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let mut new_result = result.clone();
            if !chunk_htmls[i].is_empty() {
                if let Some(v) = new_result.pointer_mut("/metadata/0/chunk_html") {
                    *v = chunk_htmls.clone()[i].clone().into()
                }
            }
            new_result
        })
        .collect();

    Ok(query)
}

pub async fn get_search_metrics_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<DatasetAnalytics, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            count(*) as total_queries,
            avg(latency) as avg_latency,
            quantile(0.99)(latency) as p99,
            quantile(0.95)(latency) as p95,
            quantile(0.5)(latency) as p50,
            round(100 * countIf(JSONExtract(query_rating, 'rating', 'Nullable(Float64)') >= 1) / count(*), 2) as total_positive_ratings,
            round(100 * countIf(JSONExtract(query_rating, 'rating', 'Nullable(Float64)') <= 0) / count(*), 2) as total_negative_ratings
        FROM search_queries
        WHERE dataset_id = ?            
         ",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_one::<DatasetAnalytics>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    Ok(clickhouse_query)
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "HeadQueryResponse")]
pub struct HeadQueryResponse {
    pub queries: Vec<HeadQueries>,
}

pub async fn get_head_queries_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    page: Option<u32>,
    clickhouse_client: &clickhouse::Client,
) -> Result<HeadQueryResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            query, 
            count(*) AS count
        FROM 
            search_queries
        WHERE dataset_id = ? AND search_queries.is_duplicate = 0",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        " GROUP BY 
            query
        ORDER BY 
            count DESC
        LIMIT 10
        OFFSET ?",
    );

    let head_queries = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<HeadQueries>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    Ok(HeadQueryResponse {
        queries: head_queries,
    })
}

pub async fn get_low_confidence_queries_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    threshold: Option<f32>,
    page: Option<u32>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchQueryResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            search_queries
        WHERE dataset_id = ? AND search_queries.is_duplicate = 0",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    if let Some(threshold) = threshold {
        query_string.push_str(
            format!(
                " 
                AND top_score < {}
                ",
                threshold
            )
            .as_str(),
        );
    }

    query_string.push_str(
        "
        ORDER BY 
            top_score ASC
        LIMIT 10
        OFFSET ?",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<SearchQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let queries: Vec<SearchQueryEvent> =
        clickhouse_query.into_iter().map(|q| q.into()).collect_vec();

    Ok(SearchQueryResponse { queries })
}

pub async fn get_no_result_queries_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    page: Option<u32>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchQueryResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            search_queries
        WHERE dataset_id = ?
        AND top_score = 0 AND search_queries.is_duplicate = 0",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        ORDER BY 
            created_at DESC
        LIMIT 10
        OFFSET ?",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<SearchQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let queries: Vec<SearchQueryEvent> =
        clickhouse_query.into_iter().map(|q| q.into()).collect_vec();

    Ok(SearchQueryResponse { queries })
}

pub async fn get_all_queries_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    sort_by: Option<SearchSortBy>,
    sort_order: Option<SortOrder>,
    has_clicks: Option<bool>,
    page: Option<u32>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchQueryResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            search_queries
        ",
    );

    if let Some(has_clicks) = has_clicks {
        if has_clicks {
            query_string
                .push_str("JOIN events ON toUUID(events.request_id) = search_queries.id AND events.event_type = 'click'")
        } else {
            query_string.push_str(
                "LEFT ANTI JOIN events ON  toUUID(events.request_id) = search_queries.id AND events.event_type = 'click'",
            )
        }
    }

    query_string.push_str("WHERE dataset_id = ? AND search_queries.is_duplicate = 0");

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(&format!(
        "
        ORDER BY 
        {} {}
        LIMIT 10
        OFFSET ?",
        sort_by.clone().unwrap_or(SearchSortBy::CreatedAt),
        sort_order.clone().unwrap_or(SortOrder::Desc)
    ));

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<SearchQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let queries: Vec<SearchQueryEvent> = clickhouse_query
        .into_iter()
        .map(|q: SearchQueryEventClickhouse| q.into())
        .collect_vec();

    Ok(SearchQueryResponse { queries })
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "QueryCountResponse")]
pub struct QueryCountResponse {
    pub total_queries: Vec<SearchTypeCount>,
}

pub async fn get_query_counts_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<QueryCountResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            search_type,
            JSONExtractString(request_params, 'search_type') as search_method, 
            COUNT(*) as search_count
        FROM 
            search_queries
        WHERE dataset_id = ?",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        GROUP BY 
            search_type, search_method
        ORDER BY 
            search_count DESC",
    );

    let result_counts = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<SearchTypeCount>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    Ok(QueryCountResponse {
        total_queries: result_counts,
    })
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "PopularFiltersResponse")]
pub struct PopularFiltersResponse {
    pub popular_filters: Vec<PopularFilters>,
}

pub async fn get_popular_filter_values_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<PopularFiltersResponse, ServiceError> {
    let mut filter_string = String::new();
    if let Some(filter) = filter {
        filter_string = filter.add_to_query(String::from(""));
    }

    let query_string = format!(
                "WITH filter_data AS (
            SELECT
                'must' AS clause,
                JSONExtractArrayRaw(JSONExtractString(request_params, 'filters'), 'must') AS conditions
            FROM search_queries
            WHERE JSONExtractString(request_params, 'filters', 'must') != '[]' AND dataset_id = '{dataset_id}'::UUID {filter_string}

            UNION ALL

            SELECT
                'should' AS clause,
                JSONExtractArrayRaw(JSONExtractString(request_params, 'filters'), 'should') AS conditions
            FROM search_queries
            WHERE JSONExtractString(request_params, 'filters', 'should') != '[]' AND dataset_id = '{dataset_id}'::UUID {filter_string}

            UNION ALL

            SELECT
                'must_not' AS clause,
                JSONExtractArrayRaw(JSONExtractString(request_params, 'filters'), 'must_not') AS conditions
            FROM search_queries
            WHERE JSONExtractString(request_params, 'filters', 'must_not') != '[]' AND dataset_id = '{dataset_id}'::UUID {filter_string}
        ),
        parsed_conditions AS (
            SELECT
                clause,
                JSONExtractString(condition, 'field') AS field,
                multiIf(
                    JSONExtractString(condition, 'match_any') != '', 'match_any',
                    JSONExtractString(condition, 'match_all') != '', 'match_all',
                    JSONExtractString(condition, 'range') != '', 'range',
                    JSONExtractString(condition, 'date_range') != '', 'date_range',
                    JSONExtractString(condition, 'geo_bounding_box') != '', 'geo_bounding_box',
                    JSONExtractString(condition, 'geo_radius') != '', 'geo_radius',
                    JSONExtractString(condition, 'geo_polygon') != '', 'geo_polygon',
                    'unknown'
                ) AS filter_type,
                JSONExtractString(condition, 'match_any') AS match_any_value,
                JSONExtractString(condition, 'match_all') AS match_all_value,
                JSONExtractKeysAndValues(condition, 'range', 'Float32') AS range_value,
                JSONExtractKeysAndValues(condition, 'date_range', 'String') AS date_range_value
            FROM filter_data
            ARRAY JOIN conditions AS condition
        ),
        aggregated_conditions AS (
            SELECT
                clause,
                field,
                filter_type,
                match_any_value,
                match_all_value,
                range_value,
                date_range_value,
                count() OVER (PARTITION BY clause, field, filter_type) AS total_count,
                count() OVER (PARTITION BY clause, field, filter_type, match_any_value) AS match_any_count,
                count() OVER (PARTITION BY clause, field, filter_type, match_all_value) AS match_all_count,
                count() OVER (PARTITION BY clause, field, filter_type, range_value) AS range_count,
                count() OVER (PARTITION BY clause, field, filter_type, date_range_value) AS date_range_count
            FROM parsed_conditions
        ),
        final_aggregation AS (
            SELECT
                clause,
                field,
                filter_type,
                any(total_count) AS count,
                arraySort(groupArray((match_any_value, match_any_count))) AS match_any_agg,
                arraySort(groupArray((match_all_value, match_all_count))) AS match_all_agg,
                arraySort(groupArray((range_value, range_count))) AS range_agg,
                arraySort(groupArray((date_range_value, date_range_count))) AS date_range_agg
            FROM aggregated_conditions
            GROUP BY clause, field, filter_type
        )
        SELECT
            clause,
            field,
            filter_type,
            count,
            CASE
                WHEN filter_type = 'match_any' THEN
                    arrayStringConcat(
                        arrayMap(x -> concat(x.1, ': ', toString(x.2)),
                            match_any_agg),
                        ', '
                    )
                WHEN filter_type = 'match_all' THEN
                    arrayStringConcat(
                        arrayMap(x -> concat(x.1, ': ', toString(x.2)),
                            match_all_agg),
                        ', '
                    )
                WHEN filter_type = 'range' THEN
                    arrayStringConcat(
                        arrayMap(x -> concat(x.1, ': ', toString(x.2)),
                            range_agg),
                        ', '
                    )
                WHEN filter_type = 'date_range' THEN
                    arrayStringConcat(
                        arrayMap(x -> concat(x.1, ': ', toString(x.2)),
                            date_range_agg),
                        ', '
                    )
                ELSE 'N/A'
            END AS common_values
        FROM final_aggregation
        ORDER BY count DESC
        LIMIT 10", dataset_id = dataset_id, filter_string = filter_string);

    let popular_filters = clickhouse_client
        .query(query_string.as_str())
        .fetch_all::<PopularFiltersClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let popular_filters: Vec<PopularFilters> =
        popular_filters.into_iter().map(|f| f.into()).collect_vec();

    Ok(PopularFiltersResponse { popular_filters })
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "SearchUsageGraphResponse")]
pub struct SearchUsageGraphResponse {
    pub total_searches: i64,
    pub points: Vec<UsageGraphPoint>,
}

pub async fn get_search_usage_graph_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchUsageGraphResponse, ServiceError> {
    let granularity = granularity.unwrap_or(Granularity::Hour);
    let interval = match granularity {
        Granularity::Second => "1 SECOND",
        Granularity::Minute => "1 MINUTE",
        Granularity::Hour => "1 HOUR",
        Granularity::Day => "1 DAY",
        Granularity::Month => "1 MONTH",
        // Add other granularities as needed
    };

    let mut query_string = format!(
        "SELECT 
            CAST(toStartOfInterval(created_at, INTERVAL {}) AS DateTime) AS time_stamp,
            count(*) AS requests
        FROM 
            search_queries
        WHERE 
            dataset_id = ?
        ",
        interval
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        GROUP BY 
            time_stamp
        ORDER BY 
            time_stamp
        LIMIT
            1000",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<UsageGraphPointClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let rps_graph: Vec<UsageGraphPoint> = clickhouse_query
        .into_iter()
        .map(|q| q.into())
        .collect::<Vec<_>>();

    let total_searches = rps_graph.iter().map(|q| q.requests).sum();

    Ok(SearchUsageGraphResponse {
        total_searches,
        points: rps_graph,
    })
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "LatencyGraphResponse")]
pub struct LatencyGraphResponse {
    pub points: Vec<SearchLatencyGraph>,
}

pub async fn get_latency_graph_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<LatencyGraphResponse, ServiceError> {
    let mut query_string = String::from(
        "WITH per_second_latency AS (
            SELECT 
                toDateTime(toUnixTimestamp(created_at) - (toUnixTimestamp(created_at) % 1)) AS second,
                avg(latency) AS latency_per_second
            FROM 
                search_queries
            WHERE 
                dataset_id = ?
        ",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(&format!(
        "
            GROUP BY 
                second
        ),
        per_interval_latency AS (
            SELECT 
                CAST(toStartOfInterval(second, INTERVAL '1 {}') AS DateTime) AS time_stamp,
                avg(latency_per_second) AS average_latency
            FROM 
                per_second_latency
            GROUP BY 
                time_stamp
        )
        SELECT 
            time_stamp,
            average_latency
        FROM 
            per_interval_latency
        ORDER BY 
            time_stamp
        LIMIT
            1000",
        granularity.clone().unwrap_or(Granularity::Hour)
    ));

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<SearchLatencyGraphClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let latency_query: Vec<SearchLatencyGraph> = clickhouse_query
        .into_iter()
        .map(|q| q.into())
        .collect::<Vec<_>>();

    Ok(LatencyGraphResponse {
        points: latency_query,
    })
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "RagQueryResponse")]
pub struct RagQueryResponse {
    pub queries: Vec<RagQueryEvent>,
}

pub async fn get_rag_queries_query(
    dataset_id: uuid::Uuid,
    filter: Option<RAGAnalyticsFilter>,
    sort_by: Option<RAGSortBy>,
    sort_order: Option<SortOrder>,
    has_clicks: Option<bool>,
    page: Option<u32>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RagQueryResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields,
        FROM 
            rag_queries
        ",
    );

    if let Some(has_clicks) = has_clicks {
        if has_clicks {
            query_string
                .push_str("JOIN events ON toUUID(events.request_id) = rag_queries.id AND events.event_type = 'click'")
        } else {
            query_string.push_str(
                "LEFT ANTI JOIN events ON  toUUID(events.request_id) = rag_queries.id AND events.event_type = 'click'",
            )
        }
    }

    query_string.push_str("WHERE dataset_id = ?");

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(&format!(
        "
        ORDER BY 
        {} {}
        LIMIT 10
        OFFSET ?",
        sort_by.clone().unwrap_or(RAGSortBy::CreatedAt),
        sort_order.clone().unwrap_or(SortOrder::Desc)
    ));

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<RagQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let queries: Vec<RagQueryEvent> = clickhouse_query.into_iter().map(|q| q.into()).collect_vec();

    Ok(RagQueryResponse { queries })
}

pub async fn get_rag_usage_query(
    dataset_id: uuid::Uuid,
    filter: Option<RAGAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RAGUsageResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            count(*) as total_queries
        FROM 
            rag_queries
        WHERE dataset_id = ?",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_one::<RAGUsageResponse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    Ok(clickhouse_query)
}

pub async fn get_rag_usage_graph_query(
    dataset_id: uuid::Uuid,
    filter: Option<RAGAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RAGUsageGraphResponse, ServiceError> {
    let granularity = granularity.unwrap_or(Granularity::Hour);
    let interval = match granularity {
        Granularity::Second => "1 SECOND",
        Granularity::Minute => "1 MINUTE",
        Granularity::Hour => "1 HOUR",
        Granularity::Day => "1 DAY",
        Granularity::Month => "1 MONTH",
    };

    let mut query_string = format!(
        "SELECT 
	        CAST(toStartOfInterval(created_at, INTERVAL {}) AS DateTime) AS time_stamp,
            count(*) AS requests
        FROM 
            rag_queries
        WHERE 
            dataset_id = ?
        ",
        interval
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        GROUP BY 
            time_stamp
        ORDER BY 
            time_stamp
        LIMIT
            1000",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<UsageGraphPointClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let rps_graph: Vec<UsageGraphPoint> = clickhouse_query
        .into_iter()
        .map(|q| q.into())
        .collect::<Vec<_>>();

    Ok(RAGUsageGraphResponse { points: rps_graph })
}

pub async fn get_rag_query(
    dataset_id: uuid::Uuid,
    request_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
) -> Result<RagQueryEvent, ServiceError> {
    let clickhouse_query = clickhouse_client
        .query("SELECT ?fields FROM rag_queries WHERE id = ? AND dataset_id = ?")
        .bind(request_id)
        .bind(dataset_id)
        .fetch_one::<RagQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let query: RagQueryEvent = clickhouse_query.into();

    Ok(query)
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "RecommendationsWithClicks")]
pub struct RecommendationsEventResponse {
    pub queries: Vec<RecommendationEvent>,
}

pub async fn get_low_confidence_recommendations_query(
    dataset_id: uuid::Uuid,
    filter: Option<RecommendationAnalyticsFilter>,
    threshold: Option<f32>,
    page: Option<u32>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RecommendationsEventResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            recommendations
        WHERE dataset_id = ?",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    if let Some(threshold) = threshold {
        query_string.push_str(
            format!(
                " 
                AND top_score < {}
                ",
                threshold
            )
            .as_str(),
        );
    }

    query_string.push_str(
        "
        ORDER BY 
            top_score ASC
        LIMIT 10
        OFFSET ?",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<RecommendationEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let queries: Vec<RecommendationEvent> =
        clickhouse_query.into_iter().map(|q| q.into()).collect_vec();

    Ok(RecommendationsEventResponse { queries })
}

pub async fn get_recommendation_query(
    dataset_id: uuid::Uuid,
    recommendation_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
) -> Result<RecommendationEvent, ServiceError> {
    let clickhouse_query = clickhouse_client
        .query("SELECT ?fields FROM recommendations WHERE id = ? AND dataset_id = ?")
        .bind(recommendation_id)
        .bind(dataset_id)
        .fetch_one::<RecommendationEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let query: RecommendationEvent = clickhouse_query.into();

    Ok(query)
}

pub async fn get_recommendation_queries_query(
    dataset_id: uuid::Uuid,
    filter: Option<RecommendationAnalyticsFilter>,
    sort_by: Option<SearchSortBy>,
    sort_order: Option<SortOrder>,
    page: Option<u32>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RecommendationsEventResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            recommendations
        WHERE dataset_id = ?",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(&format!(
        "
        ORDER BY 
        {} {}
        LIMIT 10
        OFFSET ?",
        sort_by.clone().unwrap_or(SearchSortBy::CreatedAt),
        sort_order.clone().unwrap_or(SortOrder::Desc)
    ));

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<RecommendationEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let queries: Vec<RecommendationEvent> =
        clickhouse_query.into_iter().map(|q| q.into()).collect_vec();

    Ok(RecommendationsEventResponse { queries })
}

pub async fn get_recommendation_usage_graph_query(
    dataset_id: uuid::Uuid,
    filter: Option<RecommendationAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RecommendationUsageGraphResponse, ServiceError> {
    let granularity = granularity.unwrap_or(Granularity::Hour);
    let interval = match granularity {
        Granularity::Second => "1 SECOND",
        Granularity::Minute => "1 MINUTE",
        Granularity::Hour => "1 HOUR",
        Granularity::Day => "1 DAY",
        Granularity::Month => "1 MONTH",
    };

    let mut query_string = format!(
        "SELECT 
            CAST(toStartOfInterval(created_at, INTERVAL {}) AS DateTime) AS timestamp,
            count(*) AS requests
        FROM 
            recommendations
        WHERE 
            dataset_id = ?
        ",
        interval
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        GROUP BY
            timestamp
        ORDER BY
            timestamp
        LIMIT
            1000",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<RecommendationUsageGraphPointClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let total_requests = clickhouse_query.iter().map(|q| q.requests).sum::<u64>();

    let rps_graph: Vec<RecommendationUsageGraphPoint> = clickhouse_query
        .into_iter()
        .map(|q| q.into())
        .collect::<Vec<_>>();

    Ok(RecommendationUsageGraphResponse {
        points: rps_graph,
        total_requests,
    })
}

pub async fn get_recommendations_per_user_query(
    dataset_id: uuid::Uuid,
    filter: Option<RecommendationAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RecommendationsPerUserResponse, ServiceError> {
    let granularity = granularity.unwrap_or(Granularity::Hour);
    let interval = match granularity {
        Granularity::Second => "1 SECOND",
        Granularity::Minute => "1 MINUTE",
        Granularity::Hour => "1 HOUR",
        Granularity::Day => "1 DAY",
        Granularity::Month => "1 MONTH",
    };

    let mut query_string = format!(
        "WITH recommendations_per_user AS (
            SELECT 
                CAST(toStartOfInterval(created_at, INTERVAL {}) AS DateTime) AS timestamp,
                user_id,
                count(*) AS recommendations_per_user
            FROM 
                recommendations
            WHERE 
                dataset_id = ?
        ",
        interval
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        GROUP BY
            timestamp,
            user_id
        )
        SELECT
            timestamp,
            avg(recommendations_per_user) AS avg_recommendations_per_user
        FROM
            recommendations_per_user
        GROUP BY
            timestamp
        ORDER BY
            timestamp   
        LIMIT
            1000",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<RecommendationsPerUserTimePointClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let avg_recommendations_per_user = if !clickhouse_query.is_empty() {
        clickhouse_query
            .iter()
            .map(|q| q.recommendations_per_user)
            .sum::<f64>()
            / clickhouse_query.len() as f64
    } else {
        0.0
    };

    let rpu_graph: Vec<RecommendationsPerUserTimePoint> = clickhouse_query
        .into_iter()
        .map(|q| q.into())
        .collect::<Vec<_>>();

    Ok(RecommendationsPerUserResponse {
        avg_recommendations_per_user,
        points: rpu_graph,
    })
}

pub async fn get_recommendations_ctr_rate_query(
    dataset_id: uuid::Uuid,
    filter: Option<RecommendationAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RecommendationsCTRRateResponse, ServiceError> {
    let interval = match granularity {
        Some(Granularity::Second) => "1 SECOND",
        Some(Granularity::Minute) => "1 MINUTE",
        Some(Granularity::Hour) => "1 HOUR",
        Some(Granularity::Day) => "1 DAY",
        Some(Granularity::Month) => "1 MONTH",
        _ => "1 HOUR",
    };

    let mut query_string = format!(
        "
            SELECT 
                CAST(toStartOfInterval(created_at, INTERVAL {}) AS DateTime) AS timestamp,  
                count(*) AS ctr_count
            FROM 
                events
            WHERE 
                dataset_id = ? 
                AND event_type = 'click' 
                AND event_name = 'Click'
                AND request_type = 'recommendation'
        ",
        interval
    );

    if let Some(filter) = &filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        GROUP BY
            timestamp
        ORDER BY
            timestamp
        LIMIT
            1000",
    );

    let clicks_over_time = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<ClicksOverTime>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let recs_over_time =
        get_recommendation_usage_graph_query(dataset_id, filter, granularity, clickhouse_client)
            .await?
            .points;

    let ctr_metrics = clicks_over_time
        .iter()
        .zip(recs_over_time.iter())
        .map(|(click, rec)| RecommendationsCTRRateTimePoint {
            time_stamp: click.time_stamp.to_string(),
            ctr: click.clicks as f32 / rec.requests as f32,
        })
        .collect::<Vec<_>>();

    let total_ctr = if !ctr_metrics.is_empty() {
        ctr_metrics.iter().map(|q| q.ctr).sum::<f32>() / ctr_metrics.len() as f32
    } else {
        0.0
    };

    Ok(RecommendationsCTRRateResponse {
        total_ctr,
        points: ctr_metrics,
    })
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "SearchesWithClicks")]
pub struct CTRSearchQueryWithClicksResponse {
    pub queries: Vec<SearchQueriesWithClicksCTRResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "SearchesWithoutClicks")]
pub struct CTRSearchQueryWithoutClicksResponse {
    pub queries: Vec<SearchQueriesWithoutClicksCTRResponse>,
}

pub async fn get_search_ctr_metrics_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchCTRMetrics, ServiceError> {
    let mut query_string = String::from(
        "WITH total_searches AS (
            SELECT COUNT(*) AS total
            FROM search_queries
            WHERE dataset_id = ? AND is_duplicate = 0
        ),
        metadata_values AS (
            SELECT arrayJoin(JSONExtractKeys(metadata)) AS key,
                JSONExtractFloat(metadata, key) AS value
            FROM events
            JOIN search_queries ON toUUID(events.request_id) = search_queries.id
            WHERE search_queries.dataset_id = ? AND events.event_type = 'click'
        ",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
            )
        SELECT 
            searches_with_clicks,
            (searches_with_clicks * 100.0 / total) AS percent_searches_with_click,
            ((total - searches_with_clicks) * 100.0 / total) AS percent_searches_without_click,
            avg_metadata_value
        FROM (
            SELECT 
                COUNT(*) AS searches_with_clicks,
                AVG(value) AS avg_metadata_value
            FROM metadata_values
        ) AS subquery
        CROSS JOIN total_searches
        ",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind(dataset_id)
        .fetch_one::<SearchCTRMetricsClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    Ok(clickhouse_query.into())
}

pub async fn get_searches_with_clicks_query(
    dataset_id: uuid::Uuid,
    page: Option<u32>,
    filter: Option<SearchAnalyticsFilter>,
    pool: web::Data<Pool>,
    clickhouse_client: &clickhouse::Client,
) -> Result<CTRSearchQueryWithClicksResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            search_queries.query,
            search_queries.results,
            events.dataset_id,
            events.metadata,
            events.request_id,
            events.created_at
        FROM events 
        JOIN search_queries ON toUUID(events.request_id) = search_queries.id 
        WHERE search_queries.dataset_id = ? AND search_queries.is_duplicate = 0 AND events.event_type = 'click'",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        ORDER BY 
            events.created_at DESC
        LIMIT 10
        OFFSET ?",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<SearchQueriesWithClicksCTRResponseClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let queries: Vec<SearchQueriesWithClicksCTRResponse> = join_all(
        clickhouse_query
            .into_iter()
            .map(|q| q.from_clickhouse(pool.clone())),
    )
    .await;

    Ok(CTRSearchQueryWithClicksResponse { queries })
}

pub async fn get_searches_without_clicks_query(
    dataset_id: uuid::Uuid,
    page: Option<u32>,
    filter: Option<SearchAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<CTRSearchQueryWithoutClicksResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT search_queries.query, search.id, search_queries.created_at
        FROM search_queries sq
        LEFT JOIN events cd ON sq.id = toUUID(cd.request_id) AND 
            events.event_type = 'click'
        WHERE cd.request_id = '' AND search_queries.dataset_id = ? AND search_queries.is_duplicate = 0",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        ORDER BY 
            search_queries.created_at DESC
        LIMIT 10
        OFFSET ?",
    );

    let queries = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<SearchQueriesWithoutClicksCTRResponseClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let queries: Vec<SearchQueriesWithoutClicksCTRResponse> =
        queries.into_iter().map(|q| q.into()).collect::<Vec<_>>();

    Ok(CTRSearchQueryWithoutClicksResponse { queries })
}

pub async fn get_recommendation_ctr_metrics_query(
    dataset_id: uuid::Uuid,
    filter: Option<RecommendationAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RecommendationCTRMetrics, ServiceError> {
    let mut query_string = String::from(
        "WITH total_recommendations AS (
            SELECT COUNT(*) AS total
            FROM recommendations
            WHERE dataset_id = ? 
        ),
        metadata_values AS (
            SELECT arrayJoin(JSONExtractKeys(metadata)) AS key,
                JSONExtractFloat(metadata, key) AS value
            FROM events
            JOIN recommendations ON toUUID(events.request_id) = recommendations.id
            WHERE recommendations.dataset_id = ? AND events.event_type = 'click'
        ",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
         )
        SELECT 
            recommendations_with_clicks,
            (recommendations_with_clicks * 100.0 / total) AS percent_recommendations_with_click,
            ((total - recommendations_with_clicks) * 100.0 / total) AS percent_recommendations_without_click,
            avg_metadata_value
        FROM (
            SELECT 
                COUNT(*) AS recommendations_with_clicks,
                AVG(value) AS avg_metadata_value
            FROM metadata_values
        ) AS subquery
        CROSS JOIN total_recommendations
        ",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind(dataset_id)
        .fetch_one::<RecommendationCTRMetrics>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    Ok(clickhouse_query)
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "RecommendationsWithClicks")]
pub struct CTRRecommendationsWithClicksResponse {
    pub recommendations: Vec<RecommendationsWithClicksCTRResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "RecommendationsWithoutClicks")]
pub struct CTRRecommendationsWithoutClicksResponse {
    pub recommendations: Vec<RecommendationsWithoutClicksCTRResponse>,
}

pub async fn get_recommendations_with_clicks_query(
    dataset_id: uuid::Uuid,
    page: Option<u32>,
    filter: Option<RecommendationAnalyticsFilter>,
    pool: web::Data<Pool>,
    clickhouse_client: &clickhouse::Client,
) -> Result<CTRRecommendationsWithClicksResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            recommendations.positive_ids,
            recommendations.negative_ids,
            recommendations.positive_tracking_ids,
            recommendations.negative_tracking_ids,
            recommendations.results,
            events.dataset_id,
            events.request_id,
            events.metadata,
            events.created_at
        FROM events 
        JOIN recommendations ON toUUID(events.request_id) = recommendations.id 
        WHERE recommendations.dataset_id = ? AND events.event_type = 'click'",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        ORDER BY 
            events.created_at DESC
        LIMIT 10
        OFFSET ?",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<RecommendationsWithClicksCTRResponseClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let recommendations: Vec<RecommendationsWithClicksCTRResponse> = join_all(
        clickhouse_query
            .into_iter()
            .map(|q| q.from_clickhouse(pool.clone())),
    )
    .await;

    Ok(CTRRecommendationsWithClicksResponse { recommendations })
}

pub async fn get_recommendations_without_clicks_query(
    dataset_id: uuid::Uuid,
    page: Option<u32>,
    filter: Option<RecommendationAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<CTRRecommendationsWithoutClicksResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT   
            recommendations.positive_ids,
            recommendations.negative_ids,
            recommendations.positive_tracking_ids,
            recommendations.negative_tracking_ids,
            recommendations.id,
            recommendations.created_at
        FROM recommendations r
        LEFT JOIN events cd ON r.id = toUUID(cd.request_id) AND 
            events.event_type = 'click'
        WHERE cd.request_id = '' AND recommendations.dataset_id = ?",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        ORDER BY 
            recommendations.created_at DESC
        LIMIT 10
        OFFSET ?",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<RecommendationsWithoutClicksCTRResponseClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let recommendations: Vec<RecommendationsWithoutClicksCTRResponse> =
        clickhouse_query.into_iter().map(|q| q.into()).collect();

    Ok(CTRRecommendationsWithoutClicksResponse { recommendations })
}

pub async fn get_top_datasets_query(
    data: GetTopDatasetsRequestBody,
    organization_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
    pool: web::Data<Pool>,
) -> Result<Vec<TopDatasetsResponse>, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let organization_dataset_ids = datasets_columns::datasets
        .select(datasets_columns::id)
        .filter(datasets_columns::organization_id.eq(organization_id))
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Error fetching dataset ids: {:?}", e);
            ServiceError::InternalServerError("Error fetching dataset ids".to_string())
        })?;

    let mut query_string = format!(
        "SELECT 
            dataset_id,
            COUNT(*) as total_queries
        FROM 
            {}
        WHERE 
            dataset_id IN ?",
        data.r#type
    );

    if let Some(date_range) = data.date_range {
        if let Some(gt) = &date_range.gt {
            query_string.push_str(&format!(" AND created_at > '{}'", gt));
        }
        if let Some(lt) = &date_range.lt {
            query_string.push_str(&format!(" AND created_at < '{}'", lt));
        }
        if let Some(gte) = &date_range.gte {
            query_string.push_str(&format!(" AND created_at >= '{}'", gte));
        }
        if let Some(lte) = &date_range.lte {
            query_string.push_str(&format!(" AND created_at <= '{}'", lte));
        }
    }

    query_string.push_str(
        "
        GROUP BY 
            dataset_id
        ORDER BY 
            total_queries DESC
        LIMIT 10",
    );

    let clickhouse_resp_data = clickhouse_client
        .query(query_string.as_str())
        .bind(organization_dataset_ids)
        .fetch_all::<TopDatasetsResponseClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let dataset_ids = clickhouse_resp_data
        .iter()
        .map(|x| x.dataset_id)
        .collect::<Vec<_>>();
    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;
    let dataset_id_and_tracking_ids = datasets_columns::datasets
        .select((datasets_columns::id, datasets_columns::tracking_id))
        .filter(datasets_columns::id.eq_any(dataset_ids))
        .load::<(uuid::Uuid, Option<String>)>(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Error fetching dataset ids: {:?}", e);
            ServiceError::InternalServerError("Error fetching dataset ids".to_string())
        })?;

    let response = clickhouse_resp_data
        .into_iter()
        .map(|x| {
            let mut top_dataset_resps = TopDatasetsResponse::from(x.clone());
            top_dataset_resps.dataset_tracking_id = dataset_id_and_tracking_ids
                .iter()
                .find(|(id, _)| id == &x.dataset_id)
                .and_then(|(_, tracking_id)| tracking_id.clone());
            top_dataset_resps
        })
        .collect::<Vec<_>>();

    Ok(response)
}

pub async fn get_event_by_id_query(
    dataset_id: uuid::Uuid,
    event_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
) -> Result<EventData, ServiceError> {
    let clickhouse_query = clickhouse_client
        .query("SELECT ?fields FROM events WHERE id = ? AND dataset_id = ?")
        .bind(event_id)
        .bind(dataset_id)
        .fetch_one::<EventDataClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    Ok(clickhouse_query.into())
}

pub async fn get_all_events_query(
    dataset_id: uuid::Uuid,
    page: Option<u32>,
    filter: Option<EventAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<GetEventsResponseBody, ServiceError> {
    let mut query_string = format!(
        "SELECT 
            ?fields
        FROM 
            events
        WHERE dataset_id = '{}'",
        dataset_id
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string).map_err(|e| {
            log::error!("Error adding filter to query: {:?}", e);
            ServiceError::InternalServerError("Error adding filter to query".to_string())
        })?;
    }

    query_string.push_str(
        "
        ORDER BY 
            created_at DESC
        LIMIT 10
        OFFSET ?",
    );

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<EventDataClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let events: Vec<EventData> = clickhouse_query.into_iter().map(|q| q.into()).collect_vec();

    Ok(GetEventsResponseBody { events })
}

pub async fn get_rag_query_ratings_query(
    dataset_id: uuid::Uuid,
    filter: Option<RAGAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RagQueryRatingsResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT
            round(countIf(JSONExtract(query_rating, 'rating', 'Nullable(Float64)') >= 1), 2) as total_positive_ratings,
            round(countIf(JSONExtract(query_rating, 'rating', 'Nullable(Float64)') <= 0), 2) as total_negative_ratings
        FROM rag_queries
        WHERE dataset_id = ?",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    let response = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_one::<RagQueryRatingsResponse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    Ok(response)
}

pub async fn get_topic_queries_query(
    dataset_id: uuid::Uuid,
    filter: Option<TopicAnalyticsFilter>,
    sort_by: Option<RAGSortBy>,
    sort_order: Option<SortOrder>,
    has_clicks: Option<bool>,
    page: Option<u32>,
    clickhouse_client: &clickhouse::Client,
) -> Result<TopicQueriesResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            topics.id,
            topics.name,
            topics.topic_id,
            topics.owner_id,
            topics.created_at,
            topics.updated_at,
            COUNT(rag_queries.id) as message_count,
            AVG(rag_queries.top_score) as top_score,
            AVG(rag_queries.hallucination_score) as hallucination_score,
            AVG(JSONExtract(query_rating, 'rating', 'Nullable(Float64)')) as query_rating
        FROM topics 
        JOIN rag_queries ON topics.topic_id = rag_queries.topic_id
        ",
    );

    if let Some(has_clicks) = has_clicks {
        if has_clicks {
            query_string.push_str(
                "JOIN events ON rag_queries.id = toUUID(events.request_id) AND events.event_type = 'click'",
            );
        } else {
            query_string.push_str("LEFT ANTI JOIN events ON rag_queries.id = toUUID(events.request_id) AND events.event_type = 'click'");
        }
    }

    query_string.push_str("WHERE topics.dataset_id = ? ");

    if let Some(ref filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    // Apply sorting
    let sort_direction = match sort_order {
        Some(SortOrder::Asc) => "ASC",
        _ => "DESC",
    };

    query_string.push_str("GROUP BY ALL ");

    if let Some(filter) = filter {
        query_string = filter.add_having_conditions(query_string);
    }

    let sort_by_str = match sort_by {
        Some(RAGSortBy::CreatedAt) => "topics.created_at",
        Some(RAGSortBy::TopScore) => "top_score",
        Some(RAGSortBy::HallucinationScore) => "hallucination_score",
        _ => "topics.created_at",
    };

    query_string.push_str(&format!(
        "
        ORDER BY {} {} LIMIT 10 OFFSET {}",
        sort_by_str,
        sort_direction,
        (page.unwrap_or(1) - 1) * 10
    ));

    println!("{}", query_string);

    let topics = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<TopicAnalyticsSummaryClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching topics: {:?}", e);
            ServiceError::InternalServerError("Error fetching topics".to_string())
        })?;

    Ok(TopicQueriesResponse {
        topics: topics.into_iter().map(|t| t.into()).collect(),
    })
}

pub async fn get_topic_details_query(
    topic_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
) -> Result<TopicDetailsResponse, ServiceError> {
    // Get topic details
    let topic = clickhouse_client
        .query("SELECT ?fields FROM topics WHERE topic_id = ?")
        .bind(topic_id)
        .fetch_one::<TopicQueryClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching topic: {:?}", e);
            ServiceError::InternalServerError("Error fetching topic".to_string())
        })?;

    // Get message count and timestamps
    let messages = clickhouse_client
        .query("SELECT ?fields FROM rag_queries WHERE topic_id = ? ORDER BY created_at ASC")
        .bind(topic_id)
        .fetch_all::<RagQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching messages: {:?}", e);
            ServiceError::InternalServerError("Error fetching messages".to_string())
        })?;

    let messages: Vec<RagQueryEvent> = messages.into_iter().map(|q| q.into()).collect_vec();
    Ok(TopicDetailsResponse {
        topic: topic.into(),
        messages,
    })
}

pub async fn get_topics_over_time_query(
    dataset_id: uuid::Uuid,
    filter: Option<TopicAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<TopicsOverTimeResponse, ServiceError> {
    let interval = match granularity {
        Some(Granularity::Second) => "1 SECOND",
        Some(Granularity::Minute) => "1 MINUTE",
        Some(Granularity::Hour) => "1 HOUR",
        Some(Granularity::Day) => "1 DAY",
        Some(Granularity::Month) => "1 MONTH",
        None => "1 HOUR",
    };

    let mut query_string = format!(
        "SELECT 
	        CAST(toStartOfInterval(created_at, INTERVAL {}) AS DateTime) AS time_stamp,
            count(*) AS requests
        FROM 
            topics
        JOIN rag_queries ON topics.topic_id = rag_queries.topic_id
        ",
        interval
    );

    if let Some(filter_params) = &filter {
        query_string = filter_params.add_to_query(query_string);
    }

    query_string.push_str(
        "  
        AND dataset_id = ?
        GROUP BY 
            time_stamp
        ORDER BY 
            time_stamp
        LIMIT
            1000",
    );

    let time_points = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<TopicTimePointClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching time points: {:?}", e);
            ServiceError::InternalServerError("Error fetching time points".to_string())
        })?;

    let total_topics = time_points.iter().map(|x| x.topic_count).sum();

    Ok(TopicsOverTimeResponse {
        total_topics,
        points: time_points.into_iter().map(|t| t.into()).collect(),
    })
}

pub async fn get_total_unique_users_query(
    dataset_id: uuid::Uuid,
    filter: Option<ComponentAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<TotalUniqueUsersResponse, ServiceError> {
    let interval = match granularity {
        Some(Granularity::Second) => "1 SECOND",
        Some(Granularity::Minute) => "1 MINUTE",
        Some(Granularity::Hour) => "1 HOUR",
        Some(Granularity::Day) => "1 DAY",
        Some(Granularity::Month) => "1 MONTH",
        None => "1 HOUR",
    };
    let mut query_string = format!(
        "SELECT 
            CAST(toStartOfInterval(created_at, INTERVAL {}) AS DateTime) AS time_stamp,
            count(DISTINCT user_id) AS total_unique_users
        FROM 
            events
        WHERE 
            dataset_id = ? AND user_id != ''
        ",
        interval,
    );

    if let Some(filter_params) = &filter {
        query_string = filter_params.add_to_query(query_string);
    }

    query_string.push_str(
        "  
        GROUP BY 
            time_stamp
        ORDER BY 
            time_stamp
        LIMIT
            1000",
    );

    let time_points = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<TotalUniqueUsersTimePointClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching time points: {:?}", e);
            ServiceError::InternalServerError("Error fetching time points".to_string())
        })?;

    let total_unique_users = time_points.iter().map(|t| t.unique_users).sum();

    Ok(TotalUniqueUsersResponse {
        total_unique_users,
        points: time_points.into_iter().map(|t| t.into()).collect(),
    })
}

pub async fn get_top_pages_query(
    dataset_id: uuid::Uuid,
    page: Option<u32>,
    filter: Option<ComponentAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<TopPagesResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            location,
            count(*) as count
        FROM events
        WHERE dataset_id = ? AND location != ''",
    );

    if let Some(filter_params) = &filter {
        query_string = filter_params.add_to_query(query_string);
    }

    query_string.push_str(
        "
        GROUP BY 
            location
        ORDER BY 
            count DESC
        LIMIT 10
        OFFSET ?",
    );

    let top_pages = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<TopPages>()
        .await
        .map_err(|e| {
            log::error!("Error fetching top pages: {:?}", e);
            ServiceError::InternalServerError("Error fetching top pages".to_string())
        })?;

    Ok(TopPagesResponse { top_pages })
}

pub async fn get_top_components_query(
    dataset_id: uuid::Uuid,
    page: Option<u32>,
    filter: Option<ComponentAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<TopComponentsResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            JSONExtractString(metadata, 'component_props', 'componentName') as componentName,
            count(*) as count
        FROM events
        WHERE dataset_id = ? AND componentName != ''",
    );

    if let Some(filter_params) = &filter {
        query_string = filter_params.add_to_query(query_string);
    }

    query_string.push_str(
        "
        GROUP BY 
            componentName
        ORDER BY 
            count DESC
        LIMIT 10
        OFFSET ?",
    );

    let top_components = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<TopComponents>()
        .await
        .map_err(|e| {
            log::error!("Error fetching top pages: {:?}", e);
            ServiceError::InternalServerError("Error fetching top pages".to_string())
        })?;

    Ok(TopComponentsResponse { top_components })
}

pub async fn get_component_names_query(
    dataset_id: uuid::Uuid,
    page: Option<u32>,
    clickhouse_client: &clickhouse::Client,
) -> Result<ComponentNamesResponse, ServiceError> {
    let query_string = String::from(
        " SELECT DISTINCT
            JSONExtractString(metadata, 'component_props', 'componentName') AS component_name
        FROM events
            WHERE component_name != '' 
            AND dataset_id = ?
        LIMIT 10 OFFSET ?",
    );

    let component_names = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .bind((page.unwrap_or(1) - 1) * 10)
        .fetch_all::<String>()
        .await
        .map_err(|e| {
            log::error!("Error fetching top pages: {:?}", e);
            ServiceError::InternalServerError("Error fetching top pages".to_string())
        })?;

    Ok(ComponentNamesResponse { component_names })
}

#[derive(Debug, Row, Serialize, Deserialize, ToSchema)]
pub struct ClicksOverTime {
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub time_stamp: OffsetDateTime,
    pub clicks: i64,
}

pub async fn get_ctr_metrics_over_time_query(
    dataset_id: uuid::Uuid,
    filter: Option<RAGAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<CTRMetricsOverTimeResponse, ServiceError> {
    let interval = match granularity {
        Some(Granularity::Second) => "1 SECOND",
        Some(Granularity::Minute) => "1 MINUTE",
        Some(Granularity::Hour) => "1 HOUR",
        Some(Granularity::Day) => "1 DAY",
        Some(Granularity::Month) => "1 MONTH",
        None => "1 HOUR",
    };

    let mut query_string = format!(
        "SELECT 
            CAST(toStartOfInterval(created_at, INTERVAL {}) AS DateTime) AS time_stamp,
            count(*) AS clicks
        FROM 
            events
        WHERE 
            dataset_id = ? 
            AND event_type = 'click' 
            AND event_name = 'Click'
            AND request_type = 'rag'
        ",
        interval,
    );

    if let Some(filter_params) = &filter {
        query_string = filter_params.add_to_query(query_string);
    }

    query_string.push_str(
        "
        GROUP BY 
            time_stamp
        ORDER BY 
            time_stamp
        LIMIT 1000",
    );

    let clicks_over_time = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<ClicksOverTime>()
        .await
        .map_err(|e| {
            log::error!("Error fetching ctr metrics over time: {:?}", e);
            ServiceError::InternalServerError("Error fetching ctr metrics over time".to_string())
        })?;

    let chats_over_time =
        get_rag_usage_graph_query(dataset_id, filter, granularity, clickhouse_client)
            .await?
            .points;

    let ctr_metrics_over_time: Vec<CTRMetricsOverTimePoint> = clicks_over_time
        .iter()
        .zip(chats_over_time.iter())
        .map(|(x, y)| CTRMetricsOverTimePoint {
            time_stamp: x.time_stamp.to_string(),
            ctr: x.clicks as f32 / y.requests as f32,
        })
        .collect();

    let total_ctr = if !ctr_metrics_over_time.is_empty() {
        ctr_metrics_over_time.iter().map(|x| x.ctr).sum::<f32>()
            / ctr_metrics_over_time.len() as f32
    } else {
        0.0
    };

    Ok(CTRMetricsOverTimeResponse {
        total_ctr,
        points: ctr_metrics_over_time,
    })
}

pub async fn get_messages_per_user(
    dataset_id: uuid::Uuid,
    filter: Option<RAGAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<MessagesPerUserResponse, ServiceError> {
    let interval = match granularity {
        Some(Granularity::Second) => "1 SECOND",
        Some(Granularity::Minute) => "1 MINUTE",
        Some(Granularity::Hour) => "1 HOUR",
        Some(Granularity::Day) => "1 DAY",
        Some(Granularity::Month) => "1 MONTH",
        None => "1 HOUR",
    };

    let mut query_string = format!(
        "WITH user_daily_messages AS (
            SELECT
                toStartOfInterval(created_at, INTERVAL {}) AS time_stamp,
                user_id,
                COUNT(*) AS message_count
            FROM
                rag_queries
            WHERE
                dataset_id = ?
        ",
        interval,
    );

    if let Some(filter_params) = &filter {
        query_string = filter_params.add_to_query(query_string);
    }

    query_string.push_str(
        "
           GROUP BY
                time_stamp,
                user_id
        )
        SELECT
            time_stamp,
            AVG(message_count) AS avg_messages_per_user
        FROM
            user_daily_messages
        GROUP BY
            time_stamp
        ORDER BY
            time_stamp
        LIMIT 1000",
    );

    let chats_over_time = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<MessagesPerUserTimePointClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching ctr metrics over time: {:?}", e);
            ServiceError::InternalServerError("Error fetching ctr metrics over time".to_string())
        })?;

    let avg_messages_per_user = if !chats_over_time.is_empty() {
        chats_over_time
            .iter()
            .map(|x| x.messages_per_user)
            .sum::<f64>()
            / chats_over_time.len() as f64
    } else {
        0.0
    };

    Ok(MessagesPerUserResponse {
        avg_messages_per_user,
        points: chats_over_time.into_iter().map(|x| x.into()).collect(),
    })
}

pub async fn get_search_ctr_metrics_over_time_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<CTRMetricsOverTimeResponse, ServiceError> {
    let interval = match granularity {
        Some(Granularity::Second) => "1 SECOND",
        Some(Granularity::Minute) => "1 MINUTE",
        Some(Granularity::Hour) => "1 HOUR",
        Some(Granularity::Day) => "1 DAY",
        Some(Granularity::Month) => "1 MONTH",
        None => "1 HOUR",
    };

    let mut query_string = format!(
        "SELECT 
            CAST(toStartOfInterval(created_at, INTERVAL {}) AS DateTime) AS time_stamp,
            count(*) AS clicks
        FROM 
            events
        WHERE 
            dataset_id = ? 
            AND event_type = 'click' 
            AND event_name = 'Click'
            AND request_type = 'search'
        ",
        interval,
    );

    if let Some(filter_params) = &filter {
        query_string = filter_params.add_to_query(query_string);
    }

    query_string.push_str(
        "
        GROUP BY 
            time_stamp
        ORDER BY 
            time_stamp
        LIMIT 1000",
    );

    let clicks_over_time = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<ClicksOverTime>()
        .await
        .map_err(|e| {
            log::error!("Error fetching ctr metrics over time: {:?}", e);
            ServiceError::InternalServerError("Error fetching ctr metrics over time".to_string())
        })?;

    let searches_over_time =
        get_search_usage_graph_query(dataset_id, filter, granularity, clickhouse_client)
            .await?
            .points;

    let ctr_metrics_over_time: Vec<CTRMetricsOverTimePoint> = clicks_over_time
        .iter()
        .zip(searches_over_time.iter())
        .map(|(x, y)| CTRMetricsOverTimePoint {
            time_stamp: x.time_stamp.to_string(),
            ctr: x.clicks as f32 / y.requests as f32,
        })
        .collect();

    let total_ctr = if !ctr_metrics_over_time.is_empty() {
        ctr_metrics_over_time.iter().map(|x| x.ctr).sum::<f32>()
            / ctr_metrics_over_time.len() as f32
    } else {
        0.0
    };

    Ok(CTRMetricsOverTimeResponse {
        total_ctr,
        points: ctr_metrics_over_time,
    })
}
