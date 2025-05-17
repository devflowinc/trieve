use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use time::OffsetDateTime;

use crate::data::models::{Experiment, ExperimentClickhouse, ExperimentUserAssignment};
use crate::handlers::experiment_handler::UserTreatmentResponse;
use crate::{
    errors::ServiceError,
    handlers::experiment_handler::{CreateExperimentReqBody, UpdateExperimentReqBody},
};

pub async fn create_experiment_query(
    data: CreateExperimentReqBody,
    dataset_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
) -> Result<Experiment, ServiceError> {
    let experiment: Experiment = data.to_experiment(dataset_id);
    let experiment_clickhouse: ExperimentClickhouse = experiment.clone().into();
    let mut insert = clickhouse_client
        .insert("experiments")
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    insert.write(&experiment_clickhouse).await.map_err(|e| {
        ServiceError::InternalServerError(format!("Failed to insert experiment: {}", e))
    })?;

    insert.end().await.map_err(|e| {
        ServiceError::InternalServerError(format!("Failed to end experiment insert: {}", e))
    })?;

    Ok(experiment)
}

pub async fn get_experiments_query(
    dataset_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
) -> Result<Vec<Experiment>, ServiceError> {
    let experiments = clickhouse_client
        .query("SELECT ?fields FROM experiments WHERE dataset_id = ?")
        .bind(dataset_id)
        .fetch_all()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    Ok(experiments
        .into_iter()
        .map(|e: ExperimentClickhouse| e.into())
        .collect())
}

pub async fn get_experiment_by_id_query(
    experiment_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
) -> Result<Experiment, ServiceError> {
    let experiment: ExperimentClickhouse = clickhouse_client
        .query("SELECT ?fields FROM experiments WHERE id = ? AND dataset_id = ?")
        .bind(experiment_id)
        .bind(dataset_id)
        .fetch_one()
        .await
        .map_err(|e| {
            ServiceError::NotFound(format!(
                "Experiment with id {} not found in dataset {}: {}",
                experiment_id, dataset_id, e
            ))
        })?;
    Ok(experiment.into())
}

pub async fn update_experiment_query(
    data: UpdateExperimentReqBody,
    dataset_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
) -> Result<Experiment, ServiceError> {
    let mut experiment: ExperimentClickhouse = clickhouse_client
        .query("SELECT ?fields FROM experiments WHERE id = ? AND dataset_id = ?")
        .bind(data.id)
        .bind(dataset_id)
        .fetch_one()
        .await
        .map_err(|e| {
            ServiceError::NotFound(format!(
                "Experiment with id {} not found in dataset {}: {}",
                data.id, dataset_id, e
            ))
        })?;

    let mut set_clauses: Vec<String> = Vec::new();
    let mut params: Vec<Value> = Vec::new();

    if let Some(new_name) = &data.name {
        if *new_name != experiment.name {
            set_clauses.push("name = ?".to_string());
            params.push(new_name.clone().into());
            experiment.name = new_name.clone();
        }
    }

    if let Some(config) = &data.experiment_config {
        if config.t1_name != experiment.t1_name {
            set_clauses.push("t1_name = ?".to_string());
            params.push(config.t1_name.clone().into());
            experiment.t1_name = config.t1_name.clone();
        }
        if config.t1_split != experiment.t1_split {
            set_clauses.push("t1_split = ?".to_string());
            params.push(config.t1_split.into());
            experiment.t1_split = config.t1_split;
        }
        if config.control_name != experiment.control_name {
            set_clauses.push("control_name = ?".to_string());
            params.push(config.control_name.clone().into());
            experiment.control_name = config.control_name.clone();
        }
        if config.control_split != experiment.control_split {
            set_clauses.push("control_split = ?".to_string());
            params.push(config.control_split.into());
            experiment.control_split = config.control_split;
        }
    }

    if set_clauses.is_empty() {
        return Ok(experiment.into());
    }

    let new_updated_at = OffsetDateTime::now_utc();
    set_clauses.push("updated_at = ?".to_string());

    experiment.updated_at = new_updated_at;

    let query_str = format!(
        "ALTER TABLE experiments UPDATE {} WHERE id = ? AND dataset_id = ?",
        set_clauses.join(", ")
    );

    let mut exec_query = clickhouse_client.query(&query_str);
    for param_value in params {
        exec_query = exec_query.bind(param_value);
    }
    exec_query = exec_query.bind(new_updated_at.unix_timestamp());
    exec_query = exec_query.bind(data.id);
    exec_query = exec_query.bind(dataset_id);

    exec_query.execute().await.map_err(|e| {
        ServiceError::InternalServerError(format!("Failed to update experiment: {}", e))
    })?;

    Ok(experiment.into())
}

pub async fn delete_experiment_query(
    experiment_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
) -> Result<(), ServiceError> {
    clickhouse_client
        .query("DELETE FROM experiments WHERE id = ? AND dataset_id = ?")
        .bind(experiment_id)
        .bind(dataset_id)
        .execute()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    Ok(())
}

pub async fn ab_test_query(
    experiment_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    user_id: String,
    clickhouse_client: &clickhouse::Client,
) -> Result<UserTreatmentResponse, ServiceError> {
    let existing_assignment: Option<ExperimentUserAssignment> = clickhouse_client
        .query("SELECT ?fields FROM experiment_user_assignments WHERE experiment_id = ? AND user_id = ? AND dataset_id = ?")
        .bind(experiment_id)
        .bind(user_id.clone())
        .bind(dataset_id)
        .fetch_optional()
        .await
        .map_err(|e| ServiceError::InternalServerError(format!("Error fetching user assignment: {}", e)))?;

    if let Some(assignment) = existing_assignment {
        return Ok(UserTreatmentResponse {
            treatment_name: assignment.treatment_name,
            experiment_id: assignment.experiment_id,
            user_id: assignment.user_id,
        });
    }

    let experiment =
        get_experiment_by_id_query(experiment_id, dataset_id, clickhouse_client).await?;

    let mut hasher = DefaultHasher::new();
    user_id.hash(&mut hasher);
    experiment_id.hash(&mut hasher);
    let hash_value = hasher.finish();

    let assigned_treatment_name =
        if experiment.t1_split > 0.0 && (hash_value % 100) < (experiment.t1_split * 100.0) as u64 {
            experiment.t1_name.clone()
        } else {
            experiment.control_name.clone()
        };

    let new_assignment = ExperimentUserAssignment {
        id: uuid::Uuid::new_v4(),
        experiment_id,
        user_id: user_id.clone(),
        dataset_id,
        treatment_name: assigned_treatment_name.clone(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };

    clickhouse_client
        .insert("experiment_user_assignments") // Ensure this table name matches your schema
        .map_err(|e| ServiceError::InternalServerError(format!("Insert setup failed: {}", e)))?
        .write(&new_assignment)
        .await
        .map_err(|e| {
            ServiceError::InternalServerError(format!("Error inserting user assignment: {}", e))
        })?;

    Ok(UserTreatmentResponse {
        treatment_name: assigned_treatment_name,
        experiment_id,
        user_id,
    })
}
