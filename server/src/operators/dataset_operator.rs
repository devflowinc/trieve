use crate::diesel::RunQueryDsl;
use crate::{
    data::models::{Dataset, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};

pub async fn create_dataset_query(
    new_dataset: Dataset,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl::*;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    diesel::insert_into(datasets)
        .values(&new_dataset)
        .execute(&mut conn)
        .map_err(|_| ServiceError::BadRequest("Failed to create dataset".to_string()))?;

    Ok(new_dataset)
}

pub fn get_dataset_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let organization: Dataset = datasets_columns::datasets
        .filter(datasets_columns::id.eq(id))
        .select(Dataset::as_select())
        .first(&mut conn)
        .map_err(|_| ServiceError::BadRequest("Could not find dataset".to_string()))?;

    Ok(organization)
}
