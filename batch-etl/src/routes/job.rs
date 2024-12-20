use actix_web::{delete, get, post, web, HttpResponse};

use crate::{
    errors::ServiceError,
    models::{CreateJobRequest, Job},
    operators::job::{cancel_job_query, create_job_query, get_job_output_query, get_job_query},
};

/// Create a Job
///
/// This endpoint creates a job in the Batch ETL system and OpenAI. The job is used to define the work that will be done on the data ingested into the system.
#[utoipa::path(
    post,
    path = "/job",
    tag = "Job",
    context_path = "/api",
    request_body(content = CreateJobRequest, description = "JSON request payload to create a new job", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON response payload containing the created job", body = Job),
        (status = 400, description = "Error typically due to deserialization issues", body = ServiceError),
    ),
)]
#[post("")]
pub async fn create_job(
    clickhouse_client: web::Data<clickhouse::Client>,
    job: web::Json<CreateJobRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let job = create_job_query(&job.into_inner(), &clickhouse_client).await?;

    Ok(HttpResponse::Ok().json(job))
}

/// Get a Job
///
/// This endpoint retrieves a job by its id.
#[utoipa::path(
    get,
    path = "/job/{job_id}",
    tag = "Job",
    context_path = "/api",
    params(
        ("job_id" = String, Path, description = "The id of the job you want to retrieve."),
    ),
    responses(
        (status = 200, description = "JSON response payload containing the job", body = Job),
        (status = 400, description = "Error typically due to deserialization issues", body = ServiceError),
    ),
)]
#[get("/{job_id}")]
pub async fn get_job(
    job_id: web::Path<String>,
    clickhouse_client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let job_id = job_id.into_inner();

    let job = get_job_query(&job_id, &clickhouse_client).await?;

    Ok(HttpResponse::Ok().json(job))
}

/// Cancel a Job
///
/// This endpoint cancels a job by its id.
#[utoipa::path(
    delete,
    path = "/job/{job_id}",
    tag = "Job",
    context_path = "/api",
    params(
        ("job_id" = String, Path, description = "The id of the job you want to cancel."),
    ),
    responses(
        (status = 200, description = "JSON response payload containing the canceled job", body = Job),
        (status = 400, description = "Error typically due to deserialization issues", body = ServiceError),
    ),
)]
#[delete("/{job_id}")]
pub async fn cancel_job(
    job_id: web::Path<String>,
    clickhouse_client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let job_id = job_id.into_inner();

    let job = cancel_job_query(&job_id, &clickhouse_client).await?;

    Ok(HttpResponse::Ok().json(job))
}

/// Get Job Output
///
/// This endpoint retrieves the output of a job by its id. The output is a S3 signed URL to the data that was processed by the job.
#[utoipa::path(
    get,
    path = "/output/{job_id}",
    tag = "Job",
    context_path = "/api",
    params(
        ("job_id" = String, Path, description = "The id of the job you want to retrieve the output for."),
    ),
    responses(
        (status = 200, description = "JSON response payload containing the output URL", body = String),
        (status = 400, description = "Error typically due to deserialization issues", body = ServiceError),
    ),
)]
#[get("/output/{job_id}")]
pub async fn get_job_output(
    job_id: web::Path<String>,
    clickhouse_client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let job_id = job_id.into_inner();

    let output_url = get_job_output_query(&job_id, &clickhouse_client).await?;

    Ok(HttpResponse::Ok().json(output_url))
}
