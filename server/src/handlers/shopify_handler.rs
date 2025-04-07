use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    data::models::{OrganizationWithSubAndPlan, Pool},
    errors::ServiceError,
    operators::{
        dittofeed_operator::{
            send_user_ditto_identity, DittoBatchRequest, DittoBatchRequestTypes,
            DittoTrackProperties, DittoTrackRequest,
        },
        organization_operator::get_org_users_by_id_query,
    },
};

use super::auth_handler::AdminOnly;

#[derive(ToSchema, Clone, Debug, Serialize, Deserialize)]
pub struct ShopifyCustomerEvent {
    pub organization_id: uuid::Uuid,
    pub store_name: String,
    pub event_type: String,
}

/// Send a Shopify user event
///
/// This endpoint is used to send a Shopify user event to all users in the organization.
#[utoipa::path(
    post,
    path = "/shopify/user_event",
    context_path = "/api",
    tag = "Public",
    request_body(content = ShopifyCustomerEvent, description = "The shopify customer data to add to this user", content_type = "application/json"),
    responses(
        (status = 200, description = "Public Page associated to the dataset"),
        (status = 400, description = "Service error relating to linking your organization to the Shopify store", body = ErrorResponseBody),
    ),
)]
pub async fn send_shopify_user_event(
    _user: AdminOnly,
    pool: web::Data<Pool>,
    org_plan_sub: OrganizationWithSubAndPlan,
    customer_event: web::Json<ShopifyCustomerEvent>,
) -> Result<HttpResponse, ServiceError> {
    let users = get_org_users_by_id_query(org_plan_sub.organization.id, pool).await?;

    let dittofeed_batch_request = DittoBatchRequest {
        batch: users
            .into_iter()
            .map(|user| {
                DittoBatchRequestTypes::Track(DittoTrackRequest {
                    r#type: Some("track".to_string()),
                    message_id: format!(
                        "{}-{}-{}",
                        customer_event.store_name.clone(),
                        user.id,
                        customer_event.event_type.clone()
                    ),
                    event: customer_event.event_type.clone(),
                    properties: DittoTrackProperties::DittoShopifyEvent(customer_event.clone()),
                    user_id: user.id,
                })
            })
            .collect(),
    };

    send_user_ditto_identity(dittofeed_batch_request).await?;

    Ok(HttpResponse::NoContent().finish())
}
