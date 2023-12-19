use super::card_operator::{
    find_relevant_sentence, get_collided_cards_query,
    get_metadata_and_collided_cards_from_point_ids_query, get_metadata_from_point_ids,
};
use super::model_operator::create_embedding;
use crate::data::models::{
    CardCollection, CardFileWithName, CardMetadataWithFileData, Dataset, DatasetConfiguration,
    FullTextSearchResult, User, UserDTO,
};
use crate::data::schema::{self};
use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use crate::errors::ServiceError;
use crate::get_env;
use crate::handlers::card_handler::{
    ParsedQuery, ScoreCardDTO, SearchCardData, SearchCardQueryResponseBody, SearchCollectionsData,
    SearchCollectionsResult,
};
use crate::operators::model_operator::CrossEncoder;
use crate::operators::qdrant_operator::{
    get_qdrant_connection, search_full_text_qdrant_query, search_semantic_qdrant_query,
};
use crate::{data::models::Pool, errors::DefaultError};
use actix_web::web;
use candle_core::Tensor;
use chrono::NaiveDateTime;
use diesel::{dsl::sql, sql_types::Text};
use diesel::{
    BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgTextExpressionMethods,
};
use itertools::Itertools;

use qdrant_client::qdrant::condition::ConditionOneOf::HasId;
use qdrant_client::qdrant::{
    point_id::PointIdOptions, Condition, Filter, HasIdCondition, PointId, SearchPoints,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub score: f32,
    pub point_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct SearchCardQueryResult {
    pub search_results: Vec<SearchResult>,
    pub total_card_pages: i64,
}

#[allow(clippy::too_many_arguments)]
pub async fn retrieve_qdrant_points_query(
    embedding_vector: Option<Vec<f32>>,
    page: u64,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    time_range: Option<(String, String)>,
    filters: Option<serde_json::Value>,
    parsed_query: ParsedQuery,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<SearchCardQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };

    // TODO: Talk to Qdrant team about how to force substring match on a field instead of keyword match
    // TEMPORARY: Using postgres to qdrant_point_id's for cards that match filter conditions
    // NOTE: Replacement function for native qdrant filters at https://gist.github.com/skeptrunedev/3ede217aa78d6462c5c52c63d0318764
    use crate::data::schema::card_collisions::dsl as card_collisions_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    let second_join = diesel::alias!(schema::card_metadata as second_join);
    let mut conn = pool.get().unwrap();

    let mut query = card_metadata_columns::card_metadata
        .left_outer_join(
            card_collisions_columns::card_collisions
                .on(card_metadata_columns::id.eq(card_collisions_columns::card_id)),
        )
        .left_outer_join(
            second_join.on(second_join
                .field(schema::card_metadata::qdrant_point_id)
                .eq(card_collisions_columns::collision_qdrant_id)),
        )
        .filter(card_metadata_columns::dataset_id.eq(dataset_id))
        .select((
            card_metadata_columns::qdrant_point_id,
            second_join
                .field(schema::card_metadata::qdrant_point_id)
                .nullable(),
        ))
        .distinct_on((
            card_metadata_columns::qdrant_point_id,
            second_join
                .field(schema::card_metadata::qdrant_point_id)
                .nullable(),
        ))
        .into_boxed();

    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();
    if !tag_set_inner.is_empty() {
        query = query.filter(card_metadata_columns::tag_set.ilike(format!(
            "%{}%",
            tag_set_inner.first().unwrap_or(&String::new())
        )));
    }

    for tag in tag_set_inner.iter().skip(1) {
        query = query.or_filter(card_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }

    if !link_inner.is_empty() {
        query = query.filter(card_metadata_columns::link.ilike(format!(
            "%{}%",
            link_inner.first().unwrap_or(&String::new())
        )));
    }
    for link_url in link_inner.iter().skip(1) {
        query = query.or_filter(card_metadata_columns::link.ilike(format!("%{}%", link_url)));
    }

    if let Some(time_range) = time_range {
        if time_range.0 != "null" && time_range.1 != "null" {
            query = query.filter(
                card_metadata_columns::time_stamp
                    .ge(
                        NaiveDateTime::parse_from_str(&time_range.0, "%Y-%m-%d %H:%M:%S").map_err(
                            |_| DefaultError {
                                message: "Failed to parse time range",
                            },
                        )?,
                    )
                    .and(card_metadata_columns::time_stamp.le(
                        NaiveDateTime::parse_from_str(&time_range.1, "%Y-%m-%d %H:%M:%S").map_err(
                            |_| DefaultError {
                                message: "Failed to parse time range",
                            },
                        )?,
                    )),
            );
        } else if time_range.0 != "null" {
            query = query.filter(card_metadata_columns::time_stamp.ge(
                NaiveDateTime::parse_from_str(&time_range.0, "%Y-%m-%d %H:%M:%S").map_err(
                    |_| DefaultError {
                        message: "Failed to parse time range",
                    },
                )?,
            ));
        } else if time_range.1 != "null" {
            query = query.filter(card_metadata_columns::time_stamp.le(
                NaiveDateTime::parse_from_str(&time_range.1, "%Y-%m-%d %H:%M:%S").map_err(
                    |_| DefaultError {
                        message: "Failed to parse time range",
                    },
                )?,
            ));
        }
    }

    if let Some(serde_json::Value::Object(obj)) = &filters {
        for key in obj.keys() {
            let value = obj.get(key).expect("Value should exist");
            match value {
                serde_json::Value::Array(arr) => {
                    query = query.filter(
                        sql::<Text>(&format!("card_metadata.metadata->>'{}'", key))
                            .ilike(format!("%{}%", arr.first().unwrap().as_str().unwrap_or(""))),
                    );
                    for item in arr.iter().skip(1) {
                        query = query.or_filter(
                            sql::<Text>(&format!("card_metadata.metadata->>'{}'", key))
                                .ilike(format!("%{}%", item.as_str().unwrap_or(""))),
                        );
                    }
                }
                _ => {
                    query = query.filter(
                        sql::<Text>(&format!("card_metadata.metadata->>'{}'", key))
                            .ilike(format!("%{}%", value.as_str().unwrap_or(""))),
                    );
                }
            }
        }
    }

    if let Some(quote_words) = parsed_query.quote_words {
        for word in quote_words.iter() {
            query = query.filter(card_metadata_columns::content.ilike(format!("%{}%", word)));
        }
    }

    if let Some(negated_words) = parsed_query.negated_words {
        for word in negated_words.iter() {
            query = query.filter(card_metadata_columns::content.not_ilike(format!("%{}%", word)));
        }
    }

    let matching_qdrant_point_ids: Vec<(Option<uuid::Uuid>, Option<uuid::Uuid>)> =
        query.load(&mut conn).map_err(|_| DefaultError {
            message: "Failed to load full-text searched cards",
        })?;

    let matching_point_ids: Vec<PointId> = matching_qdrant_point_ids
        .iter()
        .map(|uuid| {
            uuid.0
                .unwrap_or(uuid.1.unwrap_or(uuid::Uuid::nil()))
                .to_string()
        })
        .collect::<HashSet<String>>()
        .iter()
        .map(|uuid| (*uuid).clone().into())
        .collect::<Vec<PointId>>();

    let mut filter = Filter::default();
    filter.should.push(Condition {
        condition_one_of: Some(HasId(HasIdCondition {
            has_id: (matching_point_ids).to_vec(),
        })),
    });

    let point_ids = if let Some(embedding_vector) = embedding_vector {
        search_semantic_qdrant_query(page, filter, embedding_vector, dataset_id).await
    } else {
        search_full_text_qdrant_query(page, filter, parsed_query.query, dataset_id).await
    };

    Ok(SearchCardQueryResult {
        search_results: point_ids?,
        total_card_pages: (matching_qdrant_point_ids.len() as f64 / 10.0).ceil() as i64,
    })
}

pub async fn global_unfiltered_top_match_query(
    embedding_vector: Vec<f32>,
    dataset_id: uuid::Uuid,
) -> Result<SearchResult, DefaultError> {
    let qdrant = get_qdrant_connection().await?;

    let qdrant_collection = get_env!(
        "QDRANT_COLLECTION",
        "QDRANT_COLLECTION should be set if this is called"
    )
    .to_string();

    let mut dataset_filter = Filter::default();
    dataset_filter
        .must
        .push(Condition::matches("dataset_id", dataset_id.to_string()));

    let vector_name = match embedding_vector.len() {
        384 => "384_vectors",
        768 => "768_vectors",
        1024 => "1024_vectors",
        1536 => "1536_vectors",
        _ => {
            return Err(DefaultError {
                message: "Invalid embedding vector size",
            })
        }
    };

    let data = qdrant
        .search_points(&SearchPoints {
            collection_name: qdrant_collection,
            vector: embedding_vector,
            vector_name: Some(vector_name.to_string()),
            limit: 1,
            with_payload: None,
            filter: Some(dataset_filter),
            ..Default::default()
        })
        .await
        .map_err(|e| {
            log::error!("Failed to search points on Qdrant {:?}", e);
            DefaultError {
                message: "Failed to search points on Qdrant",
            }
        })?;

    let top_search_result: SearchResult = match data.result.first() {
        Some(point) => match point.clone().id {
            Some(point_id) => match point_id.point_id_options {
                Some(PointIdOptions::Uuid(id)) => SearchResult {
                    score: point.score,
                    point_id: uuid::Uuid::parse_str(&id).map_err(|_| DefaultError {
                        message: "Failed to parse uuid",
                    })?,
                },
                Some(PointIdOptions::Num(_)) => {
                    return Err(DefaultError {
                        message: "Failed to parse uuid",
                    })
                }
                None => {
                    return Err(DefaultError {
                        message: "Failed to parse uuid",
                    })
                }
            },
            None => {
                return Err(DefaultError {
                    message: "Failed to parse uuid",
                })
            }
        },
        // This only happens when there are no cards in the database
        None => SearchResult {
            score: 0.0,
            point_id: uuid::Uuid::nil(),
        },
    };

    Ok(top_search_result)
}

#[allow(clippy::too_many_arguments)]
pub async fn search_card_collections_query(
    embedding_vector: Vec<f32>,
    page: u64,
    pool: web::Data<Pool>,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    filters: Option<serde_json::Value>,
    collection_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    parsed_query: ParsedQuery,
) -> Result<SearchCardQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };
    use crate::data::schema::card_collection_bookmarks::dsl as card_collection_bookmarks_columns;
    use crate::data::schema::card_collisions::dsl as card_collisions_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;

    let mut conn = pool.get().unwrap();

    let mut query = card_metadata_columns::card_metadata
        .left_outer_join(
            card_collisions_columns::card_collisions
                .on(card_metadata_columns::id.eq(card_collisions_columns::card_id)),
        )
        .left_outer_join(
            card_collection_bookmarks_columns::card_collection_bookmarks.on(
                card_metadata_columns::id
                    .eq(card_collection_bookmarks_columns::card_metadata_id)
                    .and(card_collection_bookmarks_columns::collection_id.eq(collection_id)),
            ),
        )
        .select((
            card_metadata_columns::qdrant_point_id,
            card_collisions_columns::collision_qdrant_id.nullable(),
        ))
        .filter(card_metadata_columns::dataset_id.eq(dataset_id))
        .filter(card_collection_bookmarks_columns::collection_id.eq(collection_id))
        .distinct()
        .into_boxed();
    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();

    if let Some(tag) = tag_set_inner.first() {
        query = query.filter(card_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }
    for tag in tag_set_inner.iter().skip(1) {
        query = query.or_filter(card_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }

    if let Some(link_inner) = link_inner.first() {
        query = query.filter(card_metadata_columns::link.ilike(format!("%{}%", link_inner)));
    }
    for link_url in link_inner.iter().skip(1) {
        query = query.or_filter(card_metadata_columns::link.ilike(format!("%{}%", link_url)));
    }

    if let Some(serde_json::Value::Object(obj)) = &filters {
        for key in obj.keys() {
            if let Some(value) = obj.get(key) {
                match value {
                    serde_json::Value::Array(arr) => {
                        if let Some(first_val) = arr.first() {
                            if let Some(string_val) = first_val.as_str() {
                                query = query.filter(
                                    sql::<Text>(&format!("card_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }

                        for item in arr.iter().skip(1) {
                            if let Some(string_val) = item.as_str() {
                                query = query.or_filter(
                                    sql::<Text>(&format!("card_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }
                    }
                    serde_json::Value::String(string_val) => {
                        query = query.filter(
                            sql::<Text>(&format!("card_metadata.metadata->>'{}'", key))
                                .ilike(format!("%{}%", string_val)),
                        );
                    }
                    _ => (),
                }
            }
        }
    }

    if let Some(quote_words) = parsed_query.quote_words {
        for word in quote_words.iter() {
            query = query.filter(card_metadata_columns::content.ilike(format!("%{}%", word)));
        }
    }

    if let Some(negated_words) = parsed_query.negated_words {
        for word in negated_words.iter() {
            query = query.filter(card_metadata_columns::content.not_ilike(format!("%{}%", word)));
        }
    }

    let filtered_option_ids: Vec<(Option<uuid::Uuid>, Option<uuid::Uuid>)> =
        query.load(&mut conn).map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    let filtered_point_ids: &Vec<PointId> = &filtered_option_ids
        .iter()
        .map(|uuid| {
            uuid.0
                .unwrap_or(uuid.1.unwrap_or(uuid::Uuid::nil()))
                .to_string()
        })
        // remove duplicates
        .collect::<HashSet<String>>()
        .iter()
        .map(|uuid| (*uuid).clone().into())
        .collect::<Vec<PointId>>();

    let mut filter = Filter::default();
    filter.should.push(Condition {
        condition_one_of: Some(HasId(HasIdCondition {
            has_id: (filtered_point_ids).to_vec(),
        })),
    });

    let point_ids: Vec<SearchResult> =
        search_semantic_qdrant_query(page, filter, embedding_vector, dataset_id).await?;

    Ok(SearchCardQueryResult {
        search_results: point_ids,
        total_card_pages: (filtered_option_ids.len() as f64 / 10.0).ceil() as i64,
    })
}

pub fn get_metadata_query(
    card_metadata: Vec<FullTextSearchResult>,
    mut conn: r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
) -> Result<Vec<CardMetadataWithFileData>, DefaultError> {
    use crate::data::schema::card_collisions::dsl as card_collisions_columns;
    use crate::data::schema::card_files::dsl as card_files_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    use crate::data::schema::files::dsl as files_columns;
    use crate::data::schema::users::dsl as user_columns;

    let all_datas = card_metadata_columns::card_metadata
        .filter(
            card_metadata_columns::id.eq_any(
                card_metadata
                    .iter()
                    .map(|card| card.id)
                    .collect::<Vec<uuid::Uuid>>()
                    .as_slice(),
            ),
        )
        .left_outer_join(
            user_columns::users.on(card_metadata_columns::author_id.eq(user_columns::id)),
        )
        .left_outer_join(
            card_files_columns::card_files
                .on(card_metadata_columns::id.eq(card_files_columns::card_id)),
        )
        .left_outer_join(files_columns::files.on(card_files_columns::file_id.eq(files_columns::id)))
        .left_outer_join(
            card_collisions_columns::card_collisions
                .on(card_metadata_columns::id.eq(card_collisions_columns::card_id)),
        )
        .select((
            (
                card_files_columns::card_id,
                card_files_columns::file_id,
                files_columns::file_name,
            )
                .nullable(),
            (
                user_columns::id,
                user_columns::email,
                user_columns::created_at,
                user_columns::updated_at,
                user_columns::username,
                user_columns::website,
                user_columns::visible_email,
                user_columns::api_key_hash,
                user_columns::name,
            )
                .nullable(),
            (
                card_metadata_columns::id,
                card_collisions_columns::collision_qdrant_id.nullable(),
            ),
        ))
        .load::<(
            Option<CardFileWithName>,
            Option<User>,
            (uuid::Uuid, Option<uuid::Uuid>),
        )>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    #[allow(clippy::type_complexity)]
    let (file_ids, card_creators, card_collisions): (
        Vec<Option<CardFileWithName>>,
        Vec<Option<User>>,
        Vec<(uuid::Uuid, Option<uuid::Uuid>)>,
    ) = itertools::multiunzip(all_datas);

    let card_metadata_with_file_id: Vec<CardMetadataWithFileData> = card_metadata
        .into_iter()
        .map(|metadata| {
            let author = card_creators
                .iter()
                .flatten()
                .find(|user| user.id == metadata.author_id)
                .map(|user| UserDTO {
                    id: user.id,
                    username: user.username.clone(),
                    email: if user.visible_email {
                        Some(user.email.clone())
                    } else {
                        None
                    },
                    website: user.website.clone(),
                    visible_email: user.visible_email,
                    created_at: user.created_at,
                });

            let card_with_file_name = file_ids
                .iter()
                .flatten()
                .find(|file| file.card_id == metadata.id);

            let qdrant_point_id = match metadata.qdrant_point_id {
                Some(id) => id,
                None => {
                    card_collisions
                                    .iter()
                                    .find(|collision| collision.0 == metadata.id) // Match card id
                                    .expect("Qdrant point id does not exist for root card or collision")
                                    .1
                                    .expect("Collision Qdrant point id must exist if there is no root qdrant point id")
                },
            };

            CardMetadataWithFileData {
                id: metadata.id,
                content: metadata.content,
                link: metadata.link,
                tag_set: metadata.tag_set,
                author,
                qdrant_point_id,
                created_at: metadata.created_at,
                updated_at: metadata.updated_at,
                card_html: metadata.card_html,
                file_id: card_with_file_name.map(|file| file.file_id),
                file_name: card_with_file_name.map(|file| file.file_name.to_string()),
                metadata: metadata.metadata,
                tracking_id: metadata.tracking_id,
                time_stamp: metadata.time_stamp,
            }
        })
        .collect();
    Ok(card_metadata_with_file_id)
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct FullTextDocIds {
    pub doc_ids: Option<uuid::Uuid>,
    pub total_count: i64,
}

#[allow(clippy::too_many_arguments)]
pub async fn search_full_text_collection_query(
    user_query: String,
    page: u64,
    pool: web::Data<Pool>,
    filters: Option<serde_json::Value>,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    collection_id: uuid::Uuid,
    parsed_query: ParsedQuery,
    dataset_uuid: uuid::Uuid,
) -> Result<SearchCardQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };
    use crate::data::schema::card_collection_bookmarks::dsl as card_collection_bookmarks_columns;
    use crate::data::schema::card_collisions::dsl as card_collisions_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;

    let second_join = diesel::alias!(schema::card_metadata as second_join);

    let mut conn = pool.get().unwrap();
    // SELECT
    //     card_metadata.qdrant_point_id,
    //     second_join.qdrant_point_id
    // FROM
    //     card_metadata
    // LEFT OUTER JOIN card_collisions ON
    //     card_metadata.id = card_collisions.card_id
    //     AND card_metadata.private = false
    // LEFT OUTER JOIN card_metadata AS second_join ON
    //     second_join.qdrant_point_id = card_collisions.collision_qdrant_id
    //     AND second_join.private = true
    // WHERE
    //     card_metadata.private = false
    //     and (second_join.qdrant_point_id notnull or card_metadata.qdrant_point_id notnull);
    let mut query = card_metadata_columns::card_metadata
        .left_outer_join(
            card_collisions_columns::card_collisions
                .on(card_metadata_columns::id.eq(card_collisions_columns::card_id)),
        )
        .left_outer_join(
            second_join.on(second_join
                .field(schema::card_metadata::qdrant_point_id)
                .eq(card_collisions_columns::collision_qdrant_id)),
        )
        .left_outer_join(
            card_collection_bookmarks_columns::card_collection_bookmarks.on(
                card_metadata_columns::id
                    .eq(card_collection_bookmarks_columns::card_metadata_id)
                    .and(card_collection_bookmarks_columns::collection_id.eq(collection_id)),
            ),
        )
        .filter(card_collection_bookmarks_columns::collection_id.eq(collection_id))
        .filter(card_metadata_columns::dataset_id.eq(dataset_uuid))
        .select((
            card_metadata_columns::qdrant_point_id,
            second_join
                .field(schema::card_metadata::qdrant_point_id)
                .nullable(),
        ))
        .distinct_on((
            card_metadata_columns::qdrant_point_id,
            second_join
                .field(schema::card_metadata::qdrant_point_id)
                .nullable(),
        ))
        .into_boxed();

    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();

    if let Some(tag) = tag_set_inner.first() {
        query = query.filter(card_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }
    for tag in tag_set_inner.iter().skip(1) {
        query = query.or_filter(card_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }

    if let Some(link_inner) = link_inner.first() {
        query = query.filter(card_metadata_columns::link.ilike(format!("%{}%", link_inner)));
    }
    for link_url in link_inner.iter().skip(1) {
        query = query.or_filter(card_metadata_columns::link.ilike(format!("%{}%", link_url)));
    }

    if let Some(serde_json::Value::Object(obj)) = &filters {
        for key in obj.keys() {
            if let Some(value) = obj.get(key) {
                match value {
                    serde_json::Value::Array(arr) => {
                        if let Some(first_val) = arr.first() {
                            if let Some(string_val) = first_val.as_str() {
                                query = query.filter(
                                    sql::<Text>(&format!("card_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }

                        for item in arr.iter().skip(1) {
                            if let Some(string_val) = item.as_str() {
                                query = query.or_filter(
                                    sql::<Text>(&format!("card_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }
                    }
                    serde_json::Value::String(string_val) => {
                        query = query.filter(
                            sql::<Text>(&format!("card_metadata.metadata->>'{}'", key))
                                .ilike(format!("%{}%", string_val)),
                        );
                    }
                    _ => (),
                }
            }
        }
    }

    if let Some(quote_words) = parsed_query.quote_words {
        for word in quote_words.iter() {
            query = query.filter(card_metadata_columns::content.ilike(format!("%{}%", word)));
        }
    }

    if let Some(negated_words) = parsed_query.negated_words {
        for word in negated_words.iter() {
            query = query.filter(card_metadata_columns::content.not_ilike(format!("%{}%", word)));
        }
    }

    query = query.order((
        card_metadata_columns::qdrant_point_id,
        second_join.field(schema::card_metadata::qdrant_point_id),
    ));

    let matching_qdrant_point_ids: Vec<(Option<uuid::Uuid>, Option<uuid::Uuid>)> =
        query.load(&mut conn).map_err(|_| DefaultError {
            message: "Failed to load full-text searched cards",
        })?;

    let matching_point_ids: Vec<PointId> = matching_qdrant_point_ids
        .iter()
        .map(|uuid| {
            uuid.0
                .unwrap_or(uuid.1.unwrap_or(uuid::Uuid::nil()))
                .to_string()
        })
        .collect::<HashSet<String>>()
        .iter()
        .map(|uuid| (*uuid).clone().into())
        .collect::<Vec<PointId>>();

    let mut filter = Filter::default();
    filter.should.push(Condition {
        condition_one_of: Some(HasId(HasIdCondition {
            has_id: (matching_point_ids).to_vec(),
        })),
    });

    let point_ids = search_full_text_qdrant_query(page, filter, user_query, dataset_uuid).await;

    Ok(SearchCardQueryResult {
        search_results: point_ids?,
        total_card_pages: (matching_qdrant_point_ids.len() as f64 / 10.0).ceil() as i64,
    })
}

/// Retrieve cards from point ids, DOES NOT GUARD AGAINST DATASET ACCESS PERMISSIONS
pub async fn retrieve_cards_from_point_ids(
    search_card_query_results: SearchCardQueryResult,
    data: web::Json<SearchCardData>,
    pool: web::Data<Pool>,
) -> Result<SearchCardQueryResponseBody, actix_web::Error> {
    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let (metadata_cards, collided_cards) =
        get_metadata_and_collided_cards_from_point_ids_query(point_ids, pool)
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut card: CardMetadataWithFileData = match metadata_cards
                .iter()
                .find(|metadata_card| metadata_card.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_card) => metadata_card.clone(),
                None => CardMetadataWithFileData {
                    id: uuid::Uuid::default(),
                    author: None,
                    qdrant_point_id: uuid::Uuid::default(),
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    file_id: None,
                    file_name: None,
                    content: "".to_string(),
                    card_html: Some("".to_string()),
                    link: Some("".to_string()),
                    tag_set: Some("".to_string()),
                    metadata: None,
                    tracking_id: None,
                    time_stamp: None,
                },
            };

            card = find_relevant_sentence(card.clone(), data.content.clone()).unwrap_or(card);
            let mut collided_cards: Vec<CardMetadataWithFileData> = collided_cards
                .iter()
                .filter(|card| card.qdrant_id == search_result.point_id)
                .map(|card| card.metadata.clone())
                .collect();

            collided_cards.insert(0, card);

            ScoreCardDTO {
                metadata: collided_cards,
                score: search_result.score.into(),
            }
        })
        .collect();
    Ok(SearchCardQueryResponseBody {
        score_cards,
        total_card_pages: search_card_query_results.total_card_pages,
    })
}

pub async fn search_semantic_cards(
    data: web::Json<SearchCardData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchCardQueryResponseBody, actix_web::Error> {
    let embedding_vector = create_embedding(
        &data.content,
        DatasetConfiguration::from_json(dataset.configuration.clone()),
    )
    .await?;

    let search_card_query_results = retrieve_qdrant_points_query(
        Some(embedding_vector),
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        parsed_query,
        dataset.id,
        pool.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let result_cards =
        retrieve_cards_from_point_ids(search_card_query_results, data, pool.clone()).await?;

    Ok(result_cards)
}

pub async fn search_full_text_cards(
    data: web::Json<SearchCardData>,
    mut parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset_id: uuid::Uuid,
) -> Result<SearchCardQueryResponseBody, actix_web::Error> {
    parsed_query.query = parsed_query
        .query
        .split_whitespace()
        .join(" AND ")
        .replace('\"', "");

    let search_card_query_results = retrieve_qdrant_points_query(
        None,
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        parsed_query,
        dataset_id,
        pool.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let result_cards = retrieve_cards_from_point_ids(search_card_query_results, data, pool).await?;

    Ok(result_cards)
}

fn cross_encoder(
    results: Vec<ScoreCardDTO>,
    query: String,
    cross_encoder_init: web::Data<CrossEncoder>,
) -> Result<Vec<ScoreCardDTO>, actix_web::Error> {
    let paired_results = results
        .clone()
        .into_iter()
        .map(|score_card| {
            (
                query.clone(),
                score_card.metadata[0]
                    .card_html
                    .clone()
                    .unwrap_or(score_card.metadata[0].content.clone()),
            )
        })
        .collect::<Vec<(String, String)>>();

    let tokenizer = &cross_encoder_init.tokenizer;

    let tokenized_inputs = tokenizer
        .encode_batch(paired_results, false)
        .map_err(|e| ServiceError::BadRequest(format!("Could not tokenize inputs: {}", e)))?;

    let token_ids = tokenized_inputs
        .iter()
        .map(|tokens| {
            let tokens = tokens.get_ids().to_vec();
            Tensor::new(tokens.as_slice(), &candle_core::Device::Cpu)
                .map_err(|e| ServiceError::BadRequest(format!("Could not create tensor: {}", e)))
        })
        .collect::<Result<Vec<Tensor>, ServiceError>>()?;

    let token_ids = Tensor::stack(&token_ids, 0)
        .map_err(|e| ServiceError::BadRequest(format!("Could not stack tensors: {}", e)))?;

    let token_type_ids = token_ids
        .zeros_like()
        .map_err(|e| ServiceError::BadRequest(format!("Could not create tensor: {}", e)))?;

    let embeddings = cross_encoder_init
        .model
        .forward(&token_ids, &token_type_ids)
        .map_err(|e| ServiceError::BadRequest(format!("Could not run model: {}", e)))?;

    let scores = embeddings
        .to_dtype(candle_core::DType::F32)
        .unwrap()
        .to_vec2::<f32>()
        .unwrap()
        .into_iter()
        .flatten()
        .collect::<Vec<f32>>();

    let mut sim_scores_argsort: Vec<usize> = (0..scores.len()).collect();
    sim_scores_argsort.sort_by(|&a, &b| scores[b].partial_cmp(&scores[a]).unwrap());
    let mut sorted_corpus: Vec<ScoreCardDTO> = sim_scores_argsort
        .iter()
        .map(|&idx| results[idx].clone())
        .collect();

    for (result, &idx) in sorted_corpus.iter_mut().zip(&sim_scores_argsort) {
        result.score = scores[idx] as f64;
    }

    sorted_corpus.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    sorted_corpus.truncate(10);
    Ok(sorted_corpus)
}

fn reciprocal_rank_fusion(
    semantic_results: Vec<ScoreCardDTO>,
    full_text_results: Vec<ScoreCardDTO>,
    weights: Option<(f64, f64)>,
) -> Vec<ScoreCardDTO> {
    let mut fused_ranking: Vec<ScoreCardDTO> = Vec::new();
    let weights = weights.unwrap_or((1.0, 1.0));
    // Iterate through the union of the two result sets
    for mut document in full_text_results
        .clone()
        .into_iter()
        .chain(semantic_results.clone().into_iter())
        .unique_by(|card| card.metadata[0].id)
    {
        // Find the rank of the document in each result set
        let rank_semantic = semantic_results
            .iter()
            .position(|doc| doc.metadata[0].id == document.metadata[0].id);
        let rank_full_text = full_text_results
            .iter()
            .position(|doc| doc.metadata[0].id == document.metadata[0].id);

        // Combine Reciprocal Ranks using average or another strategy
        let combined_rank = weights.0 * (rank_semantic.unwrap_or(0) as f64)
            + weights.1 * (rank_full_text.unwrap_or(0) as f64);
        document.score = combined_rank;

        // Add the document ID and combined rank to the fused ranking
        fused_ranking.push(document.clone());
    }

    // Sort the fused ranking by combined rank in descending order
    fused_ranking.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    fused_ranking.truncate(10);

    fused_ranking
}

#[allow(clippy::too_many_arguments)]
pub async fn search_hybrid_cards(
    data: web::Json<SearchCardData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    cross_encoder_init: web::Data<CrossEncoder>,
    dataset: Dataset,
) -> Result<SearchCardQueryResponseBody, actix_web::Error> {
    let embedding_vector = create_embedding(
        &data.content,
        DatasetConfiguration::from_json(dataset.configuration.clone()),
    )
    .await?;
    let pool1 = pool.clone();

    let search_card_query_results = retrieve_qdrant_points_query(
        Some(embedding_vector),
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        parsed_query.clone(),
        dataset.id,
        pool.clone(),
    );

    let full_text_handler_results = search_full_text_cards(
        web::Json(data.clone()),
        parsed_query,
        page,
        pool,
        dataset.id,
    );

    let (search_card_query_results, full_text_handler_results) =
        futures::join!(search_card_query_results, full_text_handler_results);

    let search_card_query_results =
        search_card_query_results.map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let full_text_handler_results =
        full_text_handler_results.map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let (metadata_cards, collided_cards) =
        get_metadata_and_collided_cards_from_point_ids_query(point_ids, pool1)
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let semantic_score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut card: CardMetadataWithFileData = match metadata_cards
                .iter()
                .find(|metadata_card| metadata_card.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_card) => metadata_card.clone(),
                None => CardMetadataWithFileData {
                    id: uuid::Uuid::default(),
                    author: None,
                    qdrant_point_id: uuid::Uuid::default(),
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    file_id: None,
                    file_name: None,
                    content: "".to_string(),
                    card_html: Some("".to_string()),
                    link: Some("".to_string()),
                    tag_set: Some("".to_string()),
                    metadata: None,
                    tracking_id: None,
                    time_stamp: None,
                },
            };

            card = find_relevant_sentence(card.clone(), data.content.clone()).unwrap_or(card);
            let mut collided_cards: Vec<CardMetadataWithFileData> = collided_cards
                .iter()
                .filter(|card| card.qdrant_id == search_result.point_id)
                .map(|card| card.metadata.clone())
                .collect();

            collided_cards.insert(0, card);

            ScoreCardDTO {
                metadata: collided_cards,
                score: search_result.score as f64 * 0.5,
            }
        })
        .collect();

    if data.cross_encoder.unwrap_or(false) {
        let combined_results = semantic_score_cards
            .into_iter()
            .chain(full_text_handler_results.score_cards.into_iter())
            .unique_by(|score_card| score_card.metadata[0].id)
            .collect::<Vec<ScoreCardDTO>>();
        Ok(SearchCardQueryResponseBody {
            score_cards: cross_encoder(combined_results, data.content.clone(), cross_encoder_init)?,
            total_card_pages: search_card_query_results.total_card_pages,
        })
    } else if let Some(weights) = data.weights {
        if weights.0 == 1.0 {
            Ok(SearchCardQueryResponseBody {
                score_cards: semantic_score_cards,
                total_card_pages: search_card_query_results.total_card_pages,
            })
        } else if weights.1 == 1.0 {
            Ok(SearchCardQueryResponseBody {
                score_cards: full_text_handler_results.score_cards,
                total_card_pages: full_text_handler_results.total_card_pages,
            })
        } else {
            Ok(SearchCardQueryResponseBody {
                score_cards: reciprocal_rank_fusion(
                    semantic_score_cards,
                    full_text_handler_results.score_cards,
                    data.weights,
                ),
                total_card_pages: search_card_query_results.total_card_pages,
            })
        }
    } else {
        Ok(SearchCardQueryResponseBody {
            score_cards: reciprocal_rank_fusion(
                semantic_score_cards,
                full_text_handler_results.score_cards,
                data.weights,
            ),
            total_card_pages: search_card_query_results.total_card_pages,
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn search_semantic_collections(
    data: web::Json<SearchCollectionsData>,
    parsed_query: ParsedQuery,
    collection: CardCollection,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchCollectionsResult, actix_web::Error> {
    let embedding_vector: Vec<f32> = create_embedding(
        &data.content,
        DatasetConfiguration::from_json(dataset.configuration.clone()),
    )
    .await?;
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let pool3 = pool.clone();

    let search_card_query_results = search_card_collections_query(
        embedding_vector,
        page,
        pool2,
        data.link.clone(),
        data.tag_set.clone(),
        data.filters.clone(),
        data.collection_id,
        dataset.id,
        parsed_query,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let point_ids_1 = point_ids.clone();

    let metadata_cards = get_metadata_from_point_ids(point_ids, pool3)
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let collided_cards = get_collided_cards_query(point_ids_1, dataset.id, pool1)
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut card: CardMetadataWithFileData = match metadata_cards
                .iter()
                .find(|metadata_card| metadata_card.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_card) => metadata_card.clone(),
                None => CardMetadataWithFileData {
                    id: uuid::Uuid::default(),
                    author: None,
                    qdrant_point_id: uuid::Uuid::default(),
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    file_id: None,
                    file_name: None,
                    content: "".to_string(),
                    card_html: Some("".to_string()),
                    link: Some("".to_string()),
                    tag_set: Some("".to_string()),
                    metadata: None,
                    tracking_id: None,
                    time_stamp: None,
                },
            };
            card = find_relevant_sentence(card.clone(), data.content.clone()).unwrap_or(card);

            let mut collided_cards: Vec<CardMetadataWithFileData> = collided_cards
                .iter()
                .filter(|card| card.1 == search_result.point_id)
                .map(|card| card.0.clone())
                .collect();

            collided_cards.insert(0, card);
            // remove duplicates from collided cards
            let mut seen_ids = HashSet::new();
            let mut i = 0;
            while i < collided_cards.len() {
                if seen_ids.contains(&collided_cards[i].id) {
                    collided_cards.remove(i);
                } else {
                    seen_ids.insert(collided_cards[i].id);
                    i += 1;
                }
            }

            ScoreCardDTO {
                metadata: collided_cards,
                score: search_result.score.into(),
            }
        })
        .collect();

    Ok(SearchCollectionsResult {
        bookmarks: score_cards,
        collection,
        total_pages: search_card_query_results.total_card_pages,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn search_full_text_collections(
    data: web::Json<SearchCollectionsData>,
    parsed_query: ParsedQuery,
    collection: CardCollection,
    page: u64,
    pool: web::Data<Pool>,
    dataset_id: uuid::Uuid,
) -> Result<SearchCollectionsResult, actix_web::Error> {
    let data_inner = data.clone();
    let pool1 = pool.clone();

    let search_card_query_results = search_full_text_collection_query(
        data_inner.content.clone(),
        page,
        pool,
        data_inner.filters.clone(),
        data_inner.link.clone(),
        data_inner.tag_set.clone(),
        data_inner.collection_id,
        parsed_query,
        dataset_id,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let result_cards = retrieve_cards_from_point_ids(
        search_card_query_results,
        web::Json(data.clone().into()),
        pool1,
    )
    .await?;

    Ok(SearchCollectionsResult {
        bookmarks: result_cards.score_cards,
        collection,
        total_pages: result_cards.total_card_pages,
    })
}
