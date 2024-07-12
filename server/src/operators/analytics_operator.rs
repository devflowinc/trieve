use actix_web::web;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    data::models::{
        ClusterAnalyticsFilter, ClusterTopicsClickhouse, DatasetAnalytics, Granularity,
        HeadQueries, Pool, RAGAnalyticsFilter, RAGUsageResponse, RagQueryEvent,
        RagQueryEventClickhouse, RecommendationAnalyticsFilter, RecommendationEvent,
        RecommendationEventClickhouse, SearchAnalyticsFilter, SearchClusterTopics,
        SearchLatencyGraph, SearchLatencyGraphClickhouse, SearchQueryEvent,
        SearchQueryEventClickhouse, SearchRPSGraph, SearchRPSGraphClickhouse, SearchTypeCount,
        SortBy, SortOrder,
    },
    errors::ServiceError,
};

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
            AND search_queries.dataset_id = ? 
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
        .query("SELECT ?fields FROM search_queries WHERE id = ? AND dataset_id = ?")
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
        WHERE dataset_id = ?",
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
        AND top_score = 0",
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
