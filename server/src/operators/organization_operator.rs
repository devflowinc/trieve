use crate::{
    data::models::{Organization, Pool},
    errors::DefaultError,
};
use actix_web::web;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

pub fn create_organization_query(
    name: &str,
    configuration: serde_json::Value,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let new_organization = Organization::from_details(name.to_string(), configuration);

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let created_organization: Organization =
        diesel::insert_into(organizations_columns::organizations)
            .values(new_organization)
            .get_result(&mut conn)
            .map_err(|_| DefaultError {
                message: "Could not create organization, try again",
            })?;

    Ok(created_organization)
}

pub fn update_organization_query(
    id: uuid::Uuid,
    name: &str,
    configuration: serde_json::Value,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let updated_organization: Organization = diesel::update(organizations_columns::organizations)
        .filter(organizations_columns::id.eq(id))
        .set((
            organizations_columns::name.eq(name),
            organizations_columns::configuration.eq(configuration),
            organizations_columns::updated_at.eq(chrono::Utc::now().naive_local()),
        ))
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to update organization, try again",
        })?;

    Ok(updated_organization)
}

pub fn delete_organization_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    diesel::delete(organizations_columns::organizations)
        .filter(organizations_columns::id.eq(id))
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to delete organization, try again",
        })?;

    Ok(())
}

pub fn get_organization_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let organization: Organization = organizations_columns::organizations
        .filter(organizations_columns::id.eq(id))
        .select(Organization::as_select())
        .first(&mut conn)
        .map_err(|_| DefaultError {
            message: "Could not find organization, try again with a different id",
        })?;

    Ok(organization)
}
