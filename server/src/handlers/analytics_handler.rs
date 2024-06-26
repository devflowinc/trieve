use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    data::models::{
        AnalyticsFilter, ClusterTopicsClickhouse, DatasetAnalytics, DatasetAndOrgWithSubAndPlan,
        HeadQueries, SearchClusterTopics, SearchLatencyGraph, SearchLatencyGraphClickhouse,
        SearchQueryEvent, SearchQueryEventClickhouse, SearchRPSGraph, SearchRPSGraphClickhouse,
    },
    errors::ServiceError,
};

use super::auth_handler::AdminOnly;

/// Get Cluster Topics for a Dataset
///
/// This route allows you to view the top 15 topics for a dataset based on the clustering of the queries in the dataset.
#[utoipa::path(
    get,
    path = "/analytics/{dataset_id}/topics",
    context_path = "/api",
    tag = "analytics",
    responses(
        (status = 200, description = "The top 15 topics that users are searching for", body = SearchClusterTopics),

        (status = 400, description = "Service error relating to getting clusters", body = ErrorResponseBody),
    ),
     params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, Path, description = "The id of the dataset you want to get query clusters for."),

    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_overall_topics(
    dataset_id: web::Path<uuid::Uuid>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    if dataset_org_plan_sub.dataset.id != *dataset_id {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match provided dataset ID".to_string(),
        ));
    }

    let dataset_id = dataset_id.into_inner();
    let clickhouse_topics = clickhouse_client
        .query("SELECT ?fields FROM cluster_topics WHERE dataset_id = ? ORDER BY density DESC")
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

    Ok(HttpResponse::Ok().json(topics))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTopicQueries {
    pub dataset_id: uuid::Uuid,
    pub cluster_id: uuid::Uuid,
    pub page: i32,
}

/// Get Queries for a Topic
///
/// This route allows you to view the queries that are associated with a specific topic.
#[utoipa::path(
    get,
    path = "/analytics/{dataset_id}/{cluster_id}/{page}",
    context_path = "/api",
    tag = "analytics",
    responses(
        (status = 200, description = "The queries are contained in a topic sorted by distance to the centeroid", body = SearchQueryEvent),

        (status = 400, description = "Service error relating to getting queries", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, Path, description = "The id of the dataset you want to get query clusters for."),
        ("page" = i32, Query, description = "The page number to get the queries for the topic"),
        ("cluster_id" = uuid::Uuid, Path, description = "The id of the cluster you want to get queries for.")
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_queries_for_topic(
    data: web::Path<GetTopicQueries>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let dataset_id = data.dataset_id;
    let cluster_id = data.cluster_id;
    if dataset_org_plan_sub.dataset.id != dataset_id {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match provided dataset ID".to_string(),
        ));
    }

    let clickhouse_queries = clickhouse_client
        .query("SELECT ?fields FROM search_queries JOIN search_cluster_memberships ON search_queries.id = search_cluster_memberships.search_id WHERE search_cluster_memberships.cluster_id = ? AND search_queries.dataset_id = ? ORDER BY search_cluster_memberships.distance_to_centroid ASC LIMIT 15 OFFSET ?")
        .bind(cluster_id)
        .bind(dataset_id)
        .bind((data.page - 1) * 15)
        .fetch_all::<SearchQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching queries: {:?}", e);
            ServiceError::InternalServerError("Error fetching queries".to_string())
        })?;

    let queries: Vec<SearchQueryEvent> = clickhouse_queries
        .into_iter()
        .map(|q| q.into())
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(queries))
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct GetQueryRequest {
    pub dataset_id: uuid::Uuid,
    pub search_id: uuid::Uuid,
}

/// Get a Query
///
/// This route allows you to view the details of a specific query.
#[utoipa::path(
    get,
    path = "/analytics/{dataset_id}/query/{search_id}",
    context_path = "/api",
    tag = "analytics",
    responses(
        (status = 200, description = "The query that has been requested", body = SearchQueryEvent),

        (status = 400, description = "Service error relating to getting clusters", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, Path, description = "The id of the dataset you want to get the search for."),
        ("search_id" = uuid::Uuid, Path, description = "The id of the search.")
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_query(
    data: web::Path<GetQueryRequest>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let dataset_id = data.dataset_id;
    let search_id = data.search_id;
    if dataset_org_plan_sub.dataset.id != dataset_id {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match provided dataset ID".to_string(),
        ));
    }

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

    let query: SearchQueryEvent = clickhouse_query.into();

    Ok(HttpResponse::Ok().json(query))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetDatasetMetricsRequest {
    pub filter: Option<AnalyticsFilter>,
}

/// Get Search Metrics
///
/// This route allows you to get the search metrics for a dataset.
#[utoipa::path(
    post,
    path = "/analytics/{dataset_id}/metrics",
    context_path = "/api",
    tag = "analytics",
    request_body(content = GetDatasetMetricsRequest, description = "JSON request payload to filter the analytics", content_type = "application/json"),
    responses(
        (status = 200, description = "Metrics for the dataset", body = DatasetAnalytics),

        (status = 400, description = "Service error relating to getting dataset metrics", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, Path, description = "The id of the dataset you want to get search metrics for."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_search_metrics(
    dataset_id: web::Path<uuid::Uuid>,
    data: web::Json<GetDatasetMetricsRequest>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    if dataset_org_plan_sub.dataset.id != *dataset_id {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match provided dataset ID".to_string(),
        ));
    }

    let mut query_string = String::from(
        "SELECT 
            count(*) as total_queries,
            count(*) / dateDiff('second', min(created_at), max(created_at)) AS search_rps,
            avg(latency) as avg_latency,
            quantile(0.99)(latency) as p99,
            quantile(0.95)(latency) as p95,
            quantile(0.5)(latency) as p50
        FROM trieve.search_queries
        WHERE dataset_id = ?",
    );

    if let Some(filter) = &data.filter {
        query_string = filter.add_to_query(query_string);
    }

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id.into_inner())
        .fetch_one::<DatasetAnalytics>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    Ok(HttpResponse::Ok().json(clickhouse_query))
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct GetHeadQueriesRequest {
    pub filter: Option<AnalyticsFilter>,
    pub page: Option<u32>,
}

/// Get Head Queries
///
/// This route allows you to get the most common queries for a dataset.
#[utoipa::path(
    post,
    path = "/analytics/{dataset_id}/query/head",
    context_path = "/api",
    tag = "analytics",
    request_body(content = GetHeadQueriesRequest, description = "JSON request payload to filter the analytics", content_type = "application/json"),
    responses(
        (status = 200, description = "Head Queries for the dataset", body = Vec<HeadQueries>),

        (status = 400, description = "Service error relating to getting head queries", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, Path, description = "The id of the dataset you want to get head queries for."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_head_queries(
    dataset_id: web::Path<uuid::Uuid>,
    data: web::Json<GetHeadQueriesRequest>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    if dataset_org_plan_sub.dataset.id != *dataset_id {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match provided dataset ID".to_string(),
        ));
    }

    let mut query_string = String::from(
        "SELECT 
            query, 
            count(*) AS count
        FROM 
            trieve.search_queries
        WHERE dataset_id = ?",
    );

    if let Some(filter) = &data.filter {
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

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id.into_inner())
        .bind((data.page.unwrap_or(1) - 1) * 10)
        .fetch_all::<HeadQueries>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    Ok(HttpResponse::Ok().json(clickhouse_query))
}

/// Get Low Confidence Queries
///
/// This route allows you to get the queries that have the lowest confidence scores.
#[utoipa::path(
    post,
    path = "/analytics/{dataset_id}/query/low_confidence",
    context_path = "/api",
    tag = "analytics",
    request_body(content = GetHeadQueriesRequest, description = "JSON request payload to filter the analytics", content_type = "application/json"),
    responses(
        (status = 200, description = "Low Confidence Queries for the dataset", body = Vec<SearchQueryEvent>),

        (status = 400, description = "Service error relating to getting low confidence queries", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, Path, description = "The id of the dataset you want to get low confidence queries for."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_low_confidence_queries(
    dataset_id: web::Path<uuid::Uuid>,
    data: web::Json<GetHeadQueriesRequest>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    if dataset_org_plan_sub.dataset.id != *dataset_id {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match provided dataset ID".to_string(),
        ));
    }

    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            trieve.search_queries
        WHERE dataset_id = ?",
    );

    if let Some(filter) = &data.filter {
        query_string = filter.add_to_query(query_string);
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
        .bind(dataset_id.into_inner())
        .bind((data.page.unwrap_or(1) - 1) * 10)
        .fetch_all::<SearchQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let queries: Vec<SearchQueryEvent> = clickhouse_query
        .into_iter()
        .map(|q| q.into())
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(queries))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetAllQueriesRequest {
    pub filter: Option<AnalyticsFilter>,
    pub page: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Get All Search Queries
///
/// This route allows you to get all search queries and sort them.
#[utoipa::path(
    post,
    path = "/analytics/{dataset_id}/queries",
    context_path = "/api",
    tag = "analytics",
    request_body(content = GetAllQueriesRequest, description = "JSON request payload to filter the queries", content_type = "application/json"),
    responses(
        (status = 200, description = "Queries for the dataset", body = Vec<SearchQueryEvent>),

        (status = 400, description = "Service error relating to getting queries", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, Path, description = "The id of the dataset you want to get queries for."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_all_queries(
    dataset_id: web::Path<uuid::Uuid>,
    data: web::Json<GetAllQueriesRequest>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    if dataset_org_plan_sub.dataset.id != *dataset_id {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match provided dataset ID".to_string(),
        ));
    }

    let mut query_string = String::from(
        "SELECT 
            ?fields
        FROM 
            trieve.search_queries
        WHERE dataset_id = ?",
    );

    if let Some(filter) = &data.filter {
        query_string = filter.add_to_query(query_string);
    }

    query_string.push_str(&format!(
        "
        ORDER BY 
        {} {}
        LIMIT 10
        OFFSET ?",
        data.sort_by.clone().unwrap_or("created_at".to_string()),
        data.sort_order.clone().unwrap_or("DESC".to_string())
    ));

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id.into_inner())
        .bind((data.page.unwrap_or(1) - 1) * 10)
        .fetch_all::<SearchQueryEventClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let queries: Vec<SearchQueryEvent> = clickhouse_query
        .into_iter()
        .map(|q| q.into())
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(queries))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetRPSGraphRequest {
    pub filter: Option<AnalyticsFilter>,
    pub granularity: Option<String>,
}

/// Get RPS Graph
///
/// This route allows you to get the RPS graph for a dataset.
#[utoipa::path(
    post,
    path = "/analytics/{dataset_id}/rps",
    context_path = "/api",
    tag = "analytics",
    request_body(content = GetRPSGraphRequest, description = "JSON request payload to filter the analytics", content_type = "application/json"),
    responses(
        (status = 200, description = "RPS graph for the dataset", body = Vec<SearchRPSGraph>),

        (status = 400, description = "Service error relating to getting RPS graph", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, Path, description = "The id of the dataset you want to get RPS graph for."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_rps_graph(
    dataset_id: web::Path<uuid::Uuid>,
    data: web::Json<GetRPSGraphRequest>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    if dataset_org_plan_sub.dataset.id != *dataset_id {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match provided dataset ID".to_string(),
        ));
    }

    let mut query_string = String::from(
        "WITH per_second_rps AS (
            SELECT 
                toDateTime(toUnixTimestamp(created_at) - (toUnixTimestamp(created_at) % 1)) AS second,
                count(*) AS requests_per_second
            FROM 
                trieve.search_queries
            WHERE 
                dataset_id = ?
        ",
    );

    if let Some(filter) = &data.filter {
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
        data.granularity.clone().unwrap_or("hour".to_string())
    ));

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id.into_inner())
        .fetch_all::<SearchRPSGraphClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let query: Vec<SearchRPSGraph> = clickhouse_query
        .into_iter()
        .map(|q| q.into())
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(query))
}

/// Get Latency Graph
///
/// This route allows you to get the latency graph for a dataset.
#[utoipa::path(
    post,
    path = "/analytics/{dataset_id}/latency",
    context_path = "/api",
    tag = "analytics",
    request_body(content = GetRPSGraphRequest, description = "JSON request payload to filter the graph", content_type = "application/json"),
    responses(
        (status = 200, description = "latency graph for the dataset", body = Vec<SearchLatencyGraph>),

        (status = 400, description = "Service error relating to getting latency graph", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, Path, description = "The id of the dataset you want to get latency graph for."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_latency_graph(
    dataset_id: web::Path<uuid::Uuid>,
    data: web::Json<GetRPSGraphRequest>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    if dataset_org_plan_sub.dataset.id != *dataset_id {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match provided dataset ID".to_string(),
        ));
    }

    let mut query_string = String::from(
        "WITH per_second_latency AS (
            SELECT 
                toDateTime(toUnixTimestamp(created_at) - (toUnixTimestamp(created_at) % 1)) AS second,
                avg(latency) AS latency_per_second
            FROM 
                trieve.search_queries
            WHERE 
                dataset_id = ?
        ",
    );

    if let Some(filter) = &data.filter {
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
        data.granularity.clone().unwrap_or("hour".to_string())
    ));

    let clickhouse_query = clickhouse_client
        .query(query_string.as_str())
        .bind(dataset_id.into_inner())
        .fetch_all::<SearchLatencyGraphClickhouse>()
        .await
        .map_err(|e| {
            log::error!("Error fetching query: {:?}", e);
            ServiceError::InternalServerError("Error fetching query".to_string())
        })?;

    let query: Vec<SearchLatencyGraph> = clickhouse_query
        .into_iter()
        .map(|q| q.into())
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(query))
}
