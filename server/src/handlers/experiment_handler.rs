use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    data::models::{DatasetAndOrgWithSubAndPlan, Experiment},
    errors::ServiceError,
    operators::experiment_operator::{
        ab_test_query, create_experiment_query, delete_experiment_query, get_experiments_query,
        update_experiment_query,
    },
};

use super::auth_handler::AdminOnly;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExperimentConfig {
    pub t1_name: String,
    pub t1_split: f32,
    pub control_name: String,
    pub control_split: f32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateExperimentReqBody {
    pub name: String,
    pub experiment_config: ExperimentConfig,
}

impl CreateExperimentReqBody {
    pub fn to_experiment(&self, dataset_id: uuid::Uuid) -> Experiment {
        Experiment {
            id: uuid::Uuid::new_v4(),
            name: self.name.clone(),
            t1_name: self.experiment_config.t1_name.clone(),
            t1_split: self.experiment_config.t1_split,
            control_name: self.experiment_config.control_name.clone(),
            control_split: self.experiment_config.control_split,
            dataset_id,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

/// Create Experiment
///
/// Experiment will be created in the dataset specified via the TR-Dataset header. Auth'ed user must be an owner of the organization to create an experiment.
#[utoipa::path(
    post,
    path = "/experiment",
    context_path = "/api",
    tag = "Experiment",
    request_body(content = CreateExperimentReqBody, description = "JSON request payload to create a new experiment", content_type = "application/json"),
    responses(
        (status = 200, description = "Experiment created successfully", body = Experiment),
        (status = 400, description = "Service error relating to creating the experiment", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn create_experiment(
    data: web::Json<CreateExperimentReqBody>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let experiment = create_experiment_query(
        data.into_inner(),
        dataset_org_plan_sub.dataset.id,
        clickhouse_client.get_ref(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(experiment))
}

/// Get Experiments
///
/// Get all experiments for a dataset. Auth'ed user must be an owner of the organization to get experiments.
#[utoipa::path(
    get,
    path = "/experiment",
    context_path = "/api",
    tag = "Experiment",
    responses(
        (status = 200, description = "Experiments retrieved successfully", body = Vec<Experiment>),
        (status = 400, description = "Service error relating to getting the experiments", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn get_experiments(
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let experiments =
        get_experiments_query(dataset_org_plan_sub.dataset.id, clickhouse_client.get_ref()).await?;
    Ok(HttpResponse::Ok().json(experiments))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateExperimentReqBody {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub experiment_config: Option<ExperimentConfig>,
}

/// Update Experiment
///
/// Update an experiment. Auth'ed user must be an owner of the organization to update an experiment.
#[utoipa::path(
    put,
    path = "/experiment",
    context_path = "/api",
    tag = "Experiment",
    request_body(content = UpdateExperimentReqBody, description = "JSON request payload to update an experiment", content_type = "application/json"),
    responses(
        (status = 200, description = "Experiment updated successfully", body = Experiment),
        (status = 400, description = "Service error relating to updating the experiment", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn update_experiment(
    data: web::Json<UpdateExperimentReqBody>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let experiment = update_experiment_query(
        data.into_inner(),
        dataset_org_plan_sub.dataset.id,
        clickhouse_client.get_ref(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(experiment))
}

/// Delete Experiment
///
/// Delete an experiment. Auth'ed user must be an owner of the organization to delete an experiment.
#[utoipa::path(
    delete,
    path = "/experiment/{experiment_id}",
    context_path = "/api",
    tag = "Experiment",
    responses(
        (status = 204, description = "Experiment deleted successfully"),
        (status = 400, description = "Service error relating to deleting the experiment", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn delete_experiment(
    data: web::Path<uuid::Uuid>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    delete_experiment_query(
        data.into_inner(),
        dataset_org_plan_sub.dataset.id,
        clickhouse_client.get_ref(),
    )
    .await?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AbTestReqBody {
    pub experiment_id: uuid::Uuid,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserTreatmentResponse {
    pub treatment_name: String,
    #[schema(value_type = String, format = Uuid)]
    pub experiment_id: uuid::Uuid,
    pub user_id: String,
}

/// Ab Test
///
/// Get a user's treatment for an experiment. Auth'ed user must be an owner of the organization to get a user's treatment.
#[utoipa::path(
    post,
    path = "/experiment/ab-test",
    context_path = "/api",
    tag = "Experiment",
    request_body(content = AbTestReqBody, description = "JSON request payload to get a user's treatment", content_type = "application/json"),
    responses(
        (status = 200, description = "User treatment response", body = UserTreatmentResponse),
        (status = 400, description = "Service error relating to getting the user's treatment", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn ab_test(
    data: web::Json<AbTestReqBody>,
    _user: AdminOnly,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let treatment_response = ab_test_query(
        data.experiment_id,
        dataset_org_plan_sub.dataset.id,
        data.user_id.clone(),
        clickhouse_client.get_ref(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(treatment_response))
}
