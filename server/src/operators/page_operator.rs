use crate::data::models::Pool;
use crate::data::models::PublicPageConfiguration;
use crate::errors::ServiceError;
use actix_web::web;
use diesel::prelude::*;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;

pub async fn get_page_by_dataset_id(
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Option<PublicPageConfiguration>, ServiceError> {
    use crate::data::schema::public_page_configuration::dsl as public_page_configuration_table;

    let mut conn = pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok(public_page_configuration_table::public_page_configuration
        .filter(public_page_configuration_table::dataset_id.eq(dataset_id))
        .select(PublicPageConfiguration::as_select())
        .load::<PublicPageConfiguration>(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?
        .pop()
    )
}
