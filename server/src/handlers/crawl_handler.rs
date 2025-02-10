use actix_web::{web, HttpResponse};
use broccoli_queue::queue::BroccoliQueue;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    data::models::{CrawlOptions, DatasetAndOrgWithSubAndPlan, Pool},
    errors::ServiceError,
    operators::crawl_operator::{
        create_crawl_query, delete_crawl_query, get_crawl_requests_by_dataset_id_query,
        update_crawl_query,
    },
};

use super::auth_handler::AdminOnly;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateCrawlReqPayload {
    /// The crawl options to use for the crawl
    pub crawl_options: CrawlOptions,
}

/// Create a new crawl request
///
/// This endpoint is used to create a new crawl request for a dataset. The request payload should contain the crawl options to use for the crawl.
#[utoipa::path(
    post,
    path = "/crawl",
    context_path = "/api",
    tag = "Dataset",
    request_body(content = CreateCrawlReqPayload, description = "JSON request payload to create a new crawl", content_type = "application/json"),
    responses(
        (status = 200, description = "Crawl created successfully", body = CrawlRequest),
        (status = 400, description = "Service error relating to creating the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn create_crawl(
    data: web::Json<CreateCrawlReqPayload>,
    pool: web::Data<Pool>,
    broccoli_queue: web::Data<BroccoliQueue>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let data = data.into_inner();

    let crawl = create_crawl_query(
        data.crawl_options.clone(),
        pool.clone(),
        broccoli_queue.clone(),
        dataset_org_plan_sub.dataset.id,
    )
    .await?;

    Ok(HttpResponse::Ok().json(crawl))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateCrawlReqPayload {
    /// Crawl ID to update
    pub crawl_id: uuid::Uuid,
    /// The crawl options to update for the crawl
    pub crawl_options: CrawlOptions,
}

/// Update a crawl request
///
/// This endpoint is used to update an existing crawl request for a dataset. The request payload should contain the crawl id and the crawl options to update for the crawl.
#[utoipa::path(
    put,
    path = "/crawl",
    context_path = "/api",
    tag = "Dataset",
    request_body(content = UpdateCrawlReqPayload, description = "JSON request payload to update a crawl", content_type = "application/json"),
    responses(
        (status = 200, description = "Crawl updated successfully", body = CrawlRequest),
        (status = 400, description = "Service error relating to updating the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn update_crawl_request(
    data: web::Json<UpdateCrawlReqPayload>,
    pool: web::Data<Pool>,
    broccoli_queue: web::Data<BroccoliQueue>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let data = data.into_inner();

    let crawl = update_crawl_query(
        data.crawl_options.clone(),
        data.crawl_id,
        dataset_org_plan_sub.dataset.id,
        pool.clone(),
        broccoli_queue.clone(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(crawl))
}

/// Delete a crawl request
///
/// This endpoint is used to delete an existing crawl request for a dataset. The request payload should contain the crawl id to delete.
#[utoipa::path(
    delete,
    path = "/crawl/:crawl_id",
    context_path = "/api",
    tag = "Dataset",
    responses(
        (status = 204, description = "Crawl deleted successfully"),
        (status = 400, description = "Service error relating to deleting the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id to use for the request"),
        ("crawl_id" = uuid::Uuid, Path, description = "The id of the crawl to delete"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn delete_crawl_request(
    data: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let crawl_id = data.into_inner();

    delete_crawl_query(crawl_id, pool.clone()).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct GetCrawlRequestsReqPayload {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

/// Get all crawl requests for a dataset
///
/// This endpoint is used to get all crawl requests for a dataset.
#[utoipa::path(
    post,
    path = "/crawl/dataset",
    context_path = "/api",
    tag = "Dataset",
    request_body(content = GetCrawlRequestsReqPayload, description = "JSON request payload to get all crawl requests", content_type = "application/json"),
    responses(
        (status = 200, description = "Crawl requests retrieved successfully", body = Vec<CrawlRequest>),
        (status = 400, description = "Service error relating to retrieving the crawl requests", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_crawl_requests_for_dataset(
    params: web::Json<GetCrawlRequestsReqPayload>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let crawl_requests = get_crawl_requests_by_dataset_id_query(
        params.clone(),
        dataset_org_plan_sub.dataset.id,
        pool.clone(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(crawl_requests))
}
