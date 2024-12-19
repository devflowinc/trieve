use actix_web::{post, web, HttpResponse};

use crate::{
    models::CreateJobRequest,
    operators::{input::get_input_query, schema::get_schema_query},
};



#[post("")]
pub async fn create_job(
    clickhouse_client: web::Data<clickhouse::Client>,
    job: web::Json<CreateJobRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let schema = get_schema_query(&job.schema_id, &clickhouse_client).await?;
    let input = get_input_query(&job.input_id).await?;

    
    Ok(HttpResponse::Ok().finish())
}
