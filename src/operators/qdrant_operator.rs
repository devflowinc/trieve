use qdrant_client::qdrant::{
    points_selector::PointsSelectorOneOf, PointId, PointStruct, PointsIdsList, PointsSelector,
    WithPayloadSelector, WithVectorsSelector,
};
use serde_json::json;

use super::card_operator::get_qdrant_connection;
use crate::errors::ServiceError;

pub async fn create_new_qdrant_point_query(
    point_id: uuid::Uuid,
    embedding_vector: Vec<f32>,
    private: bool,
    author_id: Option<uuid::Uuid>,
) -> Result<(), actix_web::Error> {
    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let payload = match private {
        true => json!({"private": true, "authors": vec![author_id]})
            .try_into()
            .expect("A json! Value must always be a valid Payload"),
        false => json!({}).try_into().unwrap(),
    };

    let point = PointStruct::new(point_id.clone().to_string(), embedding_vector, payload);

    qdrant
        .upsert_points_blocking("debate_cards".to_string(), vec![point], None)
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed inserting card to qdrant".into()))?;

    Ok(())
}

pub async fn update_qdrant_point_private_query(
    point_id: uuid::Uuid,
    private: bool,
    author_id: Option<uuid::Uuid>,
) -> Result<(), actix_web::Error> {
    if private && author_id.is_none() {
        return Err(ServiceError::BadRequest("Private card must have an author".into()).into());
    }

    let qdrant_point_id: Vec<PointId> = vec![point_id.to_string().into()];

    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let current_point_vec = qdrant
        .get_points(
            "debate_cards",
            &qdrant_point_id,
            Some(WithVectorsSelector {
                selector_options: None,
            }),
            Some(WithPayloadSelector {
                selector_options: None,
            }),
            None,
        )
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed getting card from qdrant".into()))?
        .result;

    let current_point = match current_point_vec.first() {
        Some(point) => point,
        None => {
            return Err(ServiceError::BadRequest("Failed getting card from qdrant".into()).into())
        }
    };

    let current_private = match current_point.payload.get("private") {
        Some(private) => match private.as_bool() {
            Some(private) => private,
            None => false,
        },
        None => {
            return Err(ServiceError::BadRequest("Failed getting card from qdrant".into()).into())
        }
    };

    if !current_private {
        return Ok(());
    }

    let payload = match private {
        true => {
            let mut current_author_ids = match current_point.payload.get("authors") {
                Some(authors) => match authors.as_list() {
                    Some(authors) => authors
                        .iter()
                        .map(|author| match author.as_str() {
                            Some(author) => author.to_string(),
                            None => "".to_string(),
                        })
                        .filter(|author| author != "")
                        .collect::<Vec<String>>(),
                    None => {
                        vec![]
                    }
                },
                None => {
                    vec![]
                }
            };

            if !current_author_ids.contains(&author_id.unwrap().to_string()) {
                current_author_ids.push(author_id.unwrap().to_string());
            }

            json!({"private": true, "authors": current_author_ids})
        }
        false => json!({}),
    };

    let points_selector = PointsSelector {
        points_selector_one_of: Some(PointsSelectorOneOf::Points(PointsIdsList {
            ids: qdrant_point_id,
        })),
    };

    qdrant
        .set_payload(
            "debate-cards",
            &points_selector,
            payload
                .try_into()
                .expect("A json! value must always be a valid Payload"),
            None,
        )
        .await
        .map_err(|_err| {
            ServiceError::BadRequest("Failed updating card payload in qdrant".into())
        })?;

    Ok(())
}
