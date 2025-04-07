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
pub struct ShopifyCustomer {
    pub organization_id: uuid::Uuid,
    pub store_name: String,
}

/// Link Trieve Account to Shopify
///
/// Links your organization_id to a Shopify store
#[utoipa::path(
    post,
    path = "/shopify/link",
    context_path = "/api",
    tag = "Public",
    request_body(content = ShopifyCustomer, description = "The shopify customer data to add to this user", content_type = "application/json"),
    responses(
        (status = 200, description = "Public Page associated to the dataset"),
        (status = 400, description = "Service error relating to linking your organization to the Shopify store", body = ErrorResponseBody),
    ),
)]
pub async fn link_to_shopify(
    _user: AdminOnly,
    pool: web::Data<Pool>,
    org_plan_sub: OrganizationWithSubAndPlan,
    shopify_customer: web::Json<ShopifyCustomer>,
) -> Result<HttpResponse, ServiceError> {
    let users = get_org_users_by_id_query(org_plan_sub.organization.id, pool).await?;

    let dittofeed_batch_request = DittoBatchRequest {
        batch: users
            .into_iter()
            .map(|user| {
                DittoBatchRequestTypes::Track(DittoTrackRequest {
                    r#type: Some("track".to_string()),
                    message_id: shopify_customer.store_name.clone(),
                    event: "shopify_linked".to_string(),
                    properties: DittoTrackProperties::DittoShopifyLink(shopify_customer.clone()),
                    user_id: user.id,
                })
            })
            .collect(),
    };

    log::info!("Shopify Linked Request: {:#?}", dittofeed_batch_request);

    let response = send_user_ditto_identity(dittofeed_batch_request).await;

    log::info!("Shopify Linked Response: {:#?}", response);

    Ok(HttpResponse::NoContent().finish())
}
