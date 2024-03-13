use super::message_handler::get_topic_string;
use crate::{
    data::models::{DatasetAndOrgWithSubAndPlan, Pool, ServerDatasetConfiguration, Topic},
    errors::{DefaultError, ServiceError},
    handlers::auth_handler::LoggedUser,
    operators::topic_operator::{
        create_topic_query, delete_topic_query, get_all_topics_for_user_query,
        get_topic_for_user_query, update_topic_query,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateTopicData {
    /// The model to use for the assistant's messages. This can be any model from the openrouter model list. If no model is provided, the gpt-3.5-turbo will be used.
    pub model: Option<String>,
    /// The first message which will belong to the topic. The topic name is generated based on this message similar to how it works in the OpenAI chat UX if a name is not explicitly provided on the name request body key.
    pub first_user_message: Option<String>,
    /// The name of the topic. If this is not provided, the topic name is generated from the first_user_message.
    pub name: Option<String>,
}

/// create_topic
///
/// Create a new chat topic. Topics are attached to a user and act as a coordinator for memory of gen-AI chat sessions. We are considering refactoring this resource of the API soon.
#[utoipa::path(
    post,
    path = "/topic",
    context_path = "/api",
    tag = "topic",
    request_body(content = CreateTopicData, description = "JSON request payload to create chat topic", content_type = "application/json"),
    responses(
        (status = 200, description = "The JSON response payload containing the created topic", body = Topic),
        (status = 400, description = "Topic name empty or a service error", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn create_topic(
    data: web::Json<CreateTopicData>,
    user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let default_model = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    )
    .LLM_DEFAULT_MODEL;

    let model = data_inner.model.unwrap_or("".to_string());

    let model = if model.is_empty() {
        default_model
    } else {
        model
    };

    let first_message = data_inner.first_user_message;

    if first_message.is_none() && data_inner.name.is_none() {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "first_user_message and name must not be empty",
        }));
    }

    let topic_name = if let Some(first_user_message) = first_message {
        get_topic_string(model, first_user_message, &dataset_org_plan_sub.dataset)
            .await
            .map_err(|e| ServiceError::BadRequest(format!("Error getting topic string: {}", e)))?
    } else {
        data_inner.name.unwrap_or_default()
    };

    let new_topic = Topic::from_details(topic_name, user.id, dataset_org_plan_sub.dataset.id);
    let new_topic1 = new_topic.clone();

    let create_topic_result = create_topic_query(new_topic, &pool).await;

    match create_topic_result {
        Ok(()) => Ok(HttpResponse::Ok().json(new_topic1)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeleteTopicData {
    /// The id of the topic to target.
    pub topic_id: uuid::Uuid,
}

/// delete_topic
///
/// Delete an existing chat topic. When a topic is deleted, all associated chat messages are also deleted.
#[utoipa::path(
    delete,
    path = "/topic",
    context_path = "/api",
    tag = "topic",
    request_body(content = DeleteTopicData, description = "JSON request payload to delete a chat topic", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the topic was deleted"),
        (status = 400, description = "Service error relating to topic deletion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn delete_topic(
    data: web::Json<DeleteTopicData>,
    user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let topic_id = data_inner.topic_id;
    let pool_inner = pool.clone();

    let user_topic = get_topic_for_user_query(
        user.id,
        topic_id,
        dataset_org_plan_sub.dataset.id,
        &pool_inner,
    )
    .await;

    match user_topic {
        Ok(topic) => {
            let delete_topic_result =
                delete_topic_query(topic.id, dataset_org_plan_sub.dataset.id, &pool).await;

            match delete_topic_result {
                Ok(()) => Ok(HttpResponse::NoContent().finish()),
                Err(e) => Ok(HttpResponse::BadRequest().json(e)),
            }
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateTopicData {
    /// The id of the topic to target.
    pub topic_id: uuid::Uuid,
    /// The new name of the topic. A name is not generated from this field, it is used as-is.
    pub name: String,
}

/// update_topic
///
/// Update an existing chat topic. Currently, only the name of the topic can be updated.
#[utoipa::path(
    put,
    path = "/topic",
    context_path = "/api",
    tag = "topic",
    request_body(content = UpdateTopicData, description = "JSON request payload to update a chat topic", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the topic was updated"),
        (status = 400, description = "Service error relating to topic update", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn update_topic(
    data: web::Json<UpdateTopicData>,
    user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let topic_id = data_inner.topic_id;
    let name = data_inner.name;
    let pool_inner = pool.clone();

    if name.is_empty() {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Resolution must not be empty",
        }));
    }

    let user_topic = get_topic_for_user_query(
        user.id,
        topic_id,
        dataset_org_plan_sub.dataset.id,
        &pool_inner,
    )
    .await;

    match user_topic {
        Ok(topic) => {
            let update_topic_result =
                update_topic_query(topic.id, name, dataset_org_plan_sub.dataset.id, &pool).await;

            match update_topic_result {
                Ok(()) => Ok(HttpResponse::NoContent().finish()),
                Err(e) => Ok(HttpResponse::BadRequest().json(e)),
            }
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

/// get_all_topics_for_user
///
/// Get all topics belonging to a the auth'ed user. Soon, we plan to allow specification of the user for this route and include pagination.
#[utoipa::path(
    get,
    path = "/topic/user/{user_id}",
    context_path = "/api",
    tag = "topic",
    responses(
        (status = 200, description = "All topics belonging to a given user", body = Vec<Topic>),
        (status = 400, description = "Service error relating to topic get", body = ErrorResponseBody),
    ),
    params (
        ("user_id", description="The id of the user to get topics for"),
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_all_topics_for_user(
    user: LoggedUser,
    req_user_id: web::Path<uuid::Uuid>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    if user
        .user_orgs
        .iter()
        .any(|o| o.id == dataset_org_plan_sub.organization.id)
        && user.id != *req_user_id
        && user
            .user_orgs
            .iter()
            .find(|o| o.id == dataset_org_plan_sub.organization.id)
            .unwrap()
            .role
            < 1
    {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "User does not have enough permissions to get topics for another user",
        }));
    }

    let topics =
        get_all_topics_for_user_query(*req_user_id, dataset_org_plan_sub.dataset.id, &pool).await;

    match topics {
        Ok(topics) => Ok(HttpResponse::Ok().json(topics)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
