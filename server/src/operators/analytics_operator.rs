use actix_web::web;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    data::models::{
        ClusterAnalyticsFilter, ClusterTopicsClickhouse, DatasetAnalytics, Granularity,
        HeadQueries, Pool, RAGAnalyticsFilter, RAGUsageResponse, RagQueryEvent,
        RagQueryEventClickhouse, RecommendationAnalyticsFilter, RecommendationCTRMetrics,
        RecommendationEvent, RecommendationEventClickhouse, RecommendationsWithClicksCTRResponse,
        RecommendationsWithClicksCTRResponseClickhouse, RecommendationsWithoutClicksCTRResponse,
        RecommendationsWithoutClicksCTRResponseClickhouse, SearchAnalyticsFilter, SearchCTRMetrics,
        SearchCTRMetricsClickhouse, SearchClusterTopics, SearchLatencyGraph,
        SearchLatencyGraphClickhouse, SearchQueriesWithClicksCTRResponse,
        SearchQueriesWithClicksCTRResponseClickhouse, SearchQueriesWithoutClicksCTRResponse,
        SearchQueriesWithoutClicksCTRResponseClickhouse, SearchQueryEvent,
        SearchQueryEventClickhouse, SearchRPSGraph, SearchRPSGraphClickhouse, SearchTypeCount,
        SortBy, SortOrder,
    },
    errors::ServiceError,
    handlers::analytics_handler::CTRDataRequestBody,
};

use super::chunk_operator::get_metadata_from_tracking_id_query;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
pub struct SearchQueryResponse {
    pub queries: Vec<SearchQueryEvent>,
}

pub async fn get_queries_for_cluster_query(
    dataset_id: uuid::Uuid,
    cluster_id: uuid::Uuid,
    page: Option<u32>,
    pool: web::Data<Pool>,
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

    let queries: Vec<SearchQueryEvent> = join_all(
        clickhouse_queries
            .into_iter()
            .map(|q| q.from_clickhouse(pool.clone())),
    )
    .await;

    Ok(SearchQueryResponse { queries })
}

pub async fn get_search_query(
    dataset_id: uuid::Uuid,
    search_id: uuid::Uuid,
    pool: web::Data<Pool>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchQueryEvent, ServiceError> {
    let clickhouse_query = clickhouse_client
        .query("SELECT ?fields FROM search_queries WHERE id = ? AND dataset_id = ? AND search_queries.is_duplicate = 0")
        .bind(search_id)
        .bind(dataset_id)
        .fetch_one::<SearchQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let query: SearchQueryEvent = clickhouse_query.from_clickhouse(pool.clone()).await;

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
            count(*) / dateDiff('second', min(created_at), max(created_at)) AS search_rps,
            avg(latency) as avg_latency,
            quantile(0.99)(latency) as p99,
            quantile(0.95)(latency) as p95,
            quantile(0.5)(latency) as p50
        FROM default.search_queries
        WHERE dataset_id = ?",
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
            default.search_queries
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
    pool: web::Data<Pool>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchQueryResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            default.search_queries
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

    let queries: Vec<SearchQueryEvent> = join_all(
        clickhouse_query
            .into_iter()
            .map(|q| q.from_clickhouse(pool.clone())),
    )
    .await;

    Ok(SearchQueryResponse { queries })
}

pub async fn get_no_result_queries_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    page: Option<u32>,
    pool: web::Data<Pool>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchQueryResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            default.search_queries
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

    let queries: Vec<SearchQueryEvent> = join_all(
        clickhouse_query
            .into_iter()
            .map(|q| q.from_clickhouse(pool.clone())),
    )
    .await;

    Ok(SearchQueryResponse { queries })
}

pub async fn get_all_queries_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
    page: Option<u32>,
    pool: web::Data<Pool>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchQueryResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            default.search_queries
        WHERE dataset_id = ? AND search_queries.is_duplicate = 0",
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
        sort_by.clone().unwrap_or(SortBy::CreatedAt),
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

    let queries: Vec<SearchQueryEvent> = join_all(
        clickhouse_query
            .into_iter()
            .map(|q| q.from_clickhouse(pool.clone())),
    )
    .await;

    Ok(SearchQueryResponse { queries })
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
pub struct RPSGraphResponse {
    pub rps_points: Vec<SearchRPSGraph>,
}

pub async fn get_rps_graph_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    granularity: Option<Granularity>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RPSGraphResponse, ServiceError> {
    let mut query_string = String::from(
        "WITH per_second_rps AS (
            SELECT 
                toDateTime(toUnixTimestamp(created_at) - (toUnixTimestamp(created_at) % 1)) AS second,
                count(*) AS requests_per_second
            FROM 
                default.search_queries
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
        per_interval_rps AS (
            SELECT 
                toStartOfInterval(second, INTERVAL '1 {}') AS time_stamp,
                avg(requests_per_second) AS average_rps
            FROM 
                per_second_rps
            GROUP BY 
                time_stamp
        )
        SELECT 
            time_stamp,
            average_rps
        FROM 
            per_interval_rps
        ORDER BY 
            time_stamp
        LIMIT
            1000",
        granularity.clone().unwrap_or(Granularity::Hour)
    ));

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id)
        .fetch_all::<SearchRPSGraphClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let rps_graph: Vec<SearchRPSGraph> = clickhouse_query
        .into_iter()
        .map(|q| q.into())
        .collect::<Vec<_>>();

    Ok(RPSGraphResponse {
        rps_points: rps_graph,
    })
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LatencyGraphResponse {
    pub latency_points: Vec<SearchLatencyGraph>,
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
                default.search_queries
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
                toStartOfInterval(second, INTERVAL '1 {}') AS time_stamp,
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
        latency_points: latency_query,
    })
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RagQueryResponse {
    pub queries: Vec<RagQueryEvent>,
}

pub async fn get_rag_queries_query(
    dataset_id: uuid::Uuid,
    filter: Option<RAGAnalyticsFilter>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
    page: Option<u32>,
    pool: web::Data<Pool>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RagQueryResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            default.rag_queries
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
        sort_by.clone().unwrap_or(SortBy::CreatedAt),
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

    let queries: Vec<RagQueryEvent> = join_all(
        clickhouse_query
            .into_iter()
            .map(|q| q.from_clickhouse(pool.clone())),
    )
    .await;

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
            default.rag_queries
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecommendationsEventResponse {
    pub queries: Vec<RecommendationEvent>,
}

pub async fn get_low_confidence_recommendations_query(
    dataset_id: uuid::Uuid,
    filter: Option<RecommendationAnalyticsFilter>,
    threshold: Option<f32>,
    page: Option<u32>,
    pool: web::Data<Pool>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RecommendationsEventResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            default.recommendations
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

    let queries: Vec<RecommendationEvent> = join_all(
        clickhouse_query
            .into_iter()
            .map(|q| q.from_clickhouse(pool.clone())),
    )
    .await;

    Ok(RecommendationsEventResponse { queries })
}

pub async fn get_recommendation_queries_query(
    dataset_id: uuid::Uuid,
    filter: Option<RecommendationAnalyticsFilter>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
    page: Option<u32>,
    pool: web::Data<Pool>,
    clickhouse_client: &clickhouse::Client,
) -> Result<RecommendationsEventResponse, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            default.recommendations
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
        sort_by.clone().unwrap_or(SortBy::CreatedAt),
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

    let queries: Vec<RecommendationEvent> = join_all(
        clickhouse_query
            .into_iter()
            .map(|q| q.from_clickhouse(pool.clone())),
    )
    .await;

    Ok(RecommendationsEventResponse { queries })
}

pub async fn send_ctr_data_query(
    data: CTRDataRequestBody,
    clickhouse_client: &clickhouse::Client,
    pool: web::Data<Pool>,
    dataset_id: uuid::Uuid,
) -> Result<(), ServiceError> {
    let chunk_id = if let Some(chunk_id) = data.clicked_chunk_id {
        chunk_id
    } else if let Some(tracking_id) = data.clicked_chunk_tracking_id {
        get_metadata_from_tracking_id_query(tracking_id, dataset_id, pool)
            .await
            .map_err(|e| {
                log::error!("Error fetching metadata: {:?}", e);
                ServiceError::InternalServerError("Error fetching metadata".to_string())
            })?
            .id
    } else {
        return Err(ServiceError::BadRequest(
            "Missing tracking_id or clicked_chunk_id".to_string(),
        ));
    };

    clickhouse_client
        .query(
            "INSERT INTO default.ctr_data (id, request_id, type, chunk_id, dataset_id, position, metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, now())",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(data.reqeust_id)
        .bind(data.ctr_type)
        .bind(chunk_id)
        .bind(dataset_id)
        .bind(data.position)
        .bind(serde_json::to_string(&data.metadata.unwrap_or_default()).unwrap_or_default())
        .execute()
        .await
        .map_err(|err| {
            log::error!("Error writing to ClickHouse: {:?}", err);
            sentry::capture_message(&format!("Error writing to ClickHouse: {:?}", err), sentry::Level::Error);
            ServiceError::InternalServerError("Error writing to ClickHouse".to_string())
        })?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CTRSearchQueryWithClicksResponse {
    pub queries: Vec<SearchQueriesWithClicksCTRResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CTRSearchQueryWithoutClicksResponse {
    pub queries: Vec<SearchQueriesWithoutClicksCTRResponse>,
}

pub async fn get_search_ctr_metrics_query(
    dataset_id: uuid::Uuid,
    filter: Option<SearchAnalyticsFilter>,
    clickhouse_client: &clickhouse::Client,
) -> Result<SearchCTRMetrics, ServiceError> {
    let mut query_string = String::from(
        "SELECT 
            COUNT(*) AS searches_with_clicks,
            (COUNT(*)  / (
                SELECT COUNT(*) 
                FROM default.search_queries 
                WHERE dataset_id = ? AND is_duplicate = 0
            )) * 100.0 AS percent_searches_with_click,
            AVG(ctr_data.`position`) AS avg_position_of_click
        FROM default.ctr_data 
        JOIN default.search_queries ON ctr_data.request_id = search_queries.id 
        WHERE search_queries.dataset_id = ?",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

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
            ctr_data.chunk_id,
            ctr_data.dataset_id,
            ctr_data.position,
            ctr_data.created_at
        FROM default.ctr_data 
        JOIN default.search_queries ON ctr_data.request_id = search_queries.id 
        WHERE search_queries.dataset_id = ? AND search_queries.is_duplicate = 0",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        ORDER BY 
            ctr_data.created_at DESC
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
        "SELECT search_queries.query, search_queries.created_at
        FROM default.search_queries sq
        LEFT JOIN default.ctr_data cd ON sq.id = cd.request_id
        WHERE cd.request_id = '00000000-0000-0000-0000-000000000000' AND search_queries.dataset_id = ? AND search_queries.is_duplicate = 0",
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
        "SELECT 
            COUNT(*) AS recommendations_with_clicks,
            (COUNT(*)  / (
                SELECT COUNT(*) 
                FROM default.recommendations 
                WHERE dataset_id = ?SearchQueryResponse
            )) * 100.0 AS percent_recommendations_with_clicks,
            AVG(ctr_data.`position`) AS avg_position_of_click
        FROM default.ctr_data 
        JOIN default.recommendations ON ctr_data.request_id = recommendations.id 
        WHERE recommendations.dataset_id = ?",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

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
pub struct CTRRecommendationsWithClicksResponse {
    pub recommendations: Vec<RecommendationsWithClicksCTRResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
            ctr_data.chunk_id,
            ctr_data.dataset_id,
            ctr_data.position,
            ctr_data.created_at
        FROM default.ctr_data 
        JOIN default.recommendations ON ctr_data.request_id = recommendations.id 
        WHERE recommendations.dataset_id = ?",
    );

    if let Some(filter) = filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(
        "
        ORDER BY 
            ctr_data.created_at DESC
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
            recommendations.created_at
        FROM default.recommendations r
        LEFT JOIN default.ctr_data cd ON r.id = cd.request_id
        WHERE cd.request_id = '00000000-0000-0000-0000-000000000000' AND recommendations.dataset_id = ?",
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
