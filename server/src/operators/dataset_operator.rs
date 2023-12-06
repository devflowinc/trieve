use actix_web::web;
use diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::diesel::ExpressionMethods;

use crate::{errors::ServiceError, data::models::{Dataset, Pool}};


pub fn get_dataset_by_id(id: uuid::Uuid, pool: web::Data<Pool>) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl as dataset_columns;

    let mut conn = pool.get().unwrap();

    dataset_columns::datasets
        .filter(dataset_columns::id.eq(id))
        .first::<Dataset>(&mut conn)
        .map_err(|_| ServiceError::BadRequest("Dataset not found".to_string()))
}

pub fn get_dataset_by_name(name: String, pool: web::Data<Pool>) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl as dataset_columns;

    let mut conn = pool.get().unwrap();

    dataset_columns::datasets
        .filter(dataset_columns::name.eq(name))
        .first::<Dataset>(&mut conn)
        .map_err(|_| ServiceError::BadRequest("Dataset not found".to_string()))
}
