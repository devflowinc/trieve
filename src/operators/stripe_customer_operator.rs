use crate::diesel::prelude::*;
use crate::{data::models::StripeCustomer, errors::DefaultError};

pub fn create_stripe_customer_query(
    stripe_id: String,
    email: String,
    pool: &web::Data<Pool>,
) -> Result<StripeCustomer, DefaultError> {
    use crate::data::schema::stripe_customers::dsl::stripe_customers;

    let mut conn = pool.get().unwrap();

    let new_stripe_customer = StripeCustomer::from_details(stripe_id, email);

    let inserted_stripe_customer = diesel::insert_into(stripe_customers)
        .values(&new_stripe_customer)
        .get_result(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error inserting new stripe customer, try again".into(),
        })?;

    Ok(inserted_stripe_customer)
}
