use crate::{
    data::models::{
        DatasetAndOrgWithSubAndPlan, DatasetConfiguration, Pool, Topic, TopicQueryClickhouse,
    },
    errors::ServiceError,
    handlers::auth_handler::AdminOnly,
    operators::{
        clickhouse_operator::{ClickHouseEvent, EventQueue},
        message_operator::{create_messages_query, get_topic_messages_query, get_topic_string},
        topic_operator::{
            create_topic_query, delete_topic_query, get_all_topics_for_owner_id_query,
            get_topic_query, update_topic_query,
        },
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateTopicReqPayload {
    /// The first message which will belong to the topic. The topic name is generated based on this message similar to how it works in the OpenAI chat UX if a name is not explicitly provided on the name request body key.
    pub first_user_message: Option<String>,
    /// The name of the topic. If this is not provided, the topic name is generated from the first_user_message.
    pub name: Option<String>,
    /// The owner_id of the topic. This is typically a browser fingerprint or your user's id. It is used to group topics together for a user.
    pub owner_id: String,
    /// The referrer of the topic. This allows you to distinguish between multiple different sources from where your chats occur
    pub referrer: Option<String>,
}

/// Create Topic
///
/// Create a new chat topic. Topics are attached to a owner_id's and act as a coordinator for conversation message history of gen-AI chat sessions. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/topic",
    context_path = "/api",
    tag = "Topic",
    request_body(content = CreateTopicReqPayload, description = "JSON request payload to create chat topic", content_type = "application/json"),
    responses(
        (status = 200, description = "The JSON response payload containing the created topic", body = Topic),
        (status = 400, description = "Topic name empty or a service error", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn create_topic(
    data: web::Json<CreateTopicReqPayload>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    event_queue: web::Data<EventQueue>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let default_model =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone())
            .LLM_DEFAULT_MODEL;

    let first_message = data_inner.first_user_message;

    if first_message.is_none() && data_inner.name.is_none() {
        return Err(ServiceError::BadRequest(
            "first_user_message and name must not be empty".to_string(),
        )
        .into());
    }

    let topic_name = if let Some(first_user_message) = first_message {
        get_topic_string(
            default_model,
            first_user_message,
            &dataset_org_plan_sub.dataset,
        )
        .await
        .map_err(|e| ServiceError::BadRequest(format!("Error getting topic string: {}", e)))?
    } else {
        data_inner.name.unwrap_or_default()
    };

    let new_topic = Topic::from_details(
        topic_name,
        data_inner.owner_id,
        dataset_org_plan_sub.dataset.id,
    );
    let new_topic1 = new_topic.clone();

    create_topic_query(new_topic.clone(), &pool).await?;

    let clickhouse_topic_event = TopicQueryClickhouse {
        id: uuid::Uuid::new_v4(),
        topic_id: new_topic.id,
        dataset_id: dataset_org_plan_sub.dataset.id,
        referrer: data_inner.referrer.unwrap_or_default(),
        name: new_topic.name.clone(),
        owner_id: new_topic.owner_id.clone(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };

    event_queue
        .send(ClickHouseEvent::TopicCreateEvent(clickhouse_topic_event))
        .await;

    Ok(HttpResponse::Ok().json(new_topic1))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CloneTopicReqPayload {
    /// The topic_id to clone from
    pub topic_id: uuid::Uuid,
    /// The name of the topic. If this is not provided, the topic name is the same as the previous topic
    pub name: Option<String>,
    /// The owner_id of the topic. This is typically a browser fingerprint or your user's id. It is used to group topics together for a user.
    pub owner_id: String,
}

/// Clone Topic
///
/// Create a new chat topic from a `topic_id`. The new topic will be attched to the owner_id and act as a coordinator for conversation message history of gen-AI chat sessions. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/topic/clone",
    context_path = "/api",
    tag = "Topic",
    request_body(content = CloneTopicReqPayload, description = "JSON request payload to create chat topic", content_type = "application/json"),
    responses(
        (status = 200, description = "The JSON response payload containing the created topic", body = Topic),
        (status = 400, description = "Topic name empty or a service error", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn clone_topic(
    data: web::Json<CloneTopicReqPayload>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data = data.into_inner();

    //  get topic from topic_id
    let original_topic =
        get_topic_query(data.topic_id, dataset_org_plan_sub.dataset.id, &pool).await?;

    let topic_name = data.name.unwrap_or(original_topic.name);

    let new_topic = Topic::from_details(topic_name, data.owner_id, dataset_org_plan_sub.dataset.id);

    create_topic_query(new_topic.clone(), &pool).await?;

    let mut old_messages =
        get_topic_messages_query(original_topic.id, dataset_org_plan_sub.dataset.id, &pool).await?;

    old_messages.iter_mut().for_each(|message| {
        message.topic_id = new_topic.id;
        message.id = uuid::Uuid::new_v4();
    });

    create_messages_query(old_messages, &pool).await?;

    Ok(HttpResponse::Ok().json(new_topic))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeleteTopicData {
    /// The id of the topic to target.
    pub topic_id: uuid::Uuid,
}

/// Delete Topic
///
/// Delete an existing chat topic. When a topic is deleted, all associated chat messages are also deleted. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    delete,
    path = "/topic/{topic_id}",
    context_path = "/api",
    tag = "Topic",
    responses(
        (status = 204, description = "Confirmation that the topic was deleted"),
        (status = 400, description = "Service error relating to topic deletion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("topic_id" = uuid, Path, description = "The id of the topic you want to delete."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn delete_topic(
    topic_id: web::Path<uuid::Uuid>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = topic_id.into_inner();

    delete_topic_query(topic_id, dataset_org_plan_sub.dataset.id, &pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateTopicReqPayload {
    /// The id of the topic to target.
    pub topic_id: uuid::Uuid,
    /// The new name of the topic. A name is not generated from this field, it is used as-is.
    pub name: String,
}

/// Update Topic
///
/// Update an existing chat topic. Currently, only the name of the topic can be updated. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    put,
    path = "/topic",
    context_path = "/api",
    tag = "Topic",
    request_body(content = UpdateTopicReqPayload, description = "JSON request payload to update a chat topic", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the topic was updated"),
        (status = 400, description = "Service error relating to topic update", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn update_topic(
    data: web::Json<UpdateTopicReqPayload>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let topic_id = data_inner.topic_id;
    let name = data_inner.name;

    if name.is_empty() {
        return Err(ServiceError::BadRequest("Topic name must not be empty".to_string()).into());
    }

    update_topic_query(topic_id, name, dataset_org_plan_sub.dataset.id, &pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Get All Topics for Owner ID
///
/// Get all topics belonging to an arbitary owner_id. This is useful for managing message history and chat sessions. It is common to use a browser fingerprint or your user's id as the owner_id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    get,
    path = "/topic/owner/{owner_id}",
    context_path = "/api",
    tag = "Topic",
    responses(
        (status = 200, description = "All topics belonging to a given owner_id", body = Vec<Topic>),
        (status = 400, description = "Service error relating to getting topics for the owner_id", body = ErrorResponseBody),
    ),
    params (
        ("owner_id", description="The owner_id to get topics of; A common approach is to use a browser fingerprint or your user's id"),
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_all_topics_for_owner_id(
    _user: AdminOnly,
    owner_id: web::Path<String>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let topics = get_all_topics_for_owner_id_query(
        owner_id.to_string(),
        dataset_org_plan_sub.dataset.id,
        &pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(topics))
}
