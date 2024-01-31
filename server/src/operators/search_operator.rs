use super::chunk_operator::{
    find_relevant_sentence, get_metadata_and_collided_chunks_from_point_ids_query,
    get_metadata_from_point_ids,
};
use super::model_operator::{create_embedding, cross_encoder};
use crate::data::models::{
    ChunkFileWithName, ChunkGroup, ChunkMetadataWithFileData, Dataset, FullTextSearchResult,
    ServerDatasetConfiguration, User, UserDTO,
};
use crate::data::schema::{self};
use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use crate::errors::ServiceError;
use crate::get_env;
use crate::handlers::chunk_handler::{
    ParsedQuery, ScoreChunkDTO, SearchChunkData, SearchChunkQueryResponseBody, SearchGroupsData,
    SearchGroupsResult,
};
use crate::operators::qdrant_operator::{
    get_qdrant_connection, search_full_text_qdrant_query, search_semantic_qdrant_query,
};
use crate::{data::models::Pool, errors::DefaultError};
use actix_web::web;
use dateparser::DateTimeUtc;
use diesel::{dsl::sql, sql_types::Text};
use diesel::{
    BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgTextExpressionMethods,
};
use itertools::Itertools;

use qdrant_client::qdrant::condition::ConditionOneOf::HasId;
use qdrant_client::qdrant::Range;
use qdrant_client::qdrant::{
    point_id::PointIdOptions, Condition, Filter, HasIdCondition, PointId, SearchPoints,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::f32::consts::E;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub score: f32,
    pub point_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchChunkQueryResult {
    pub search_results: Vec<SearchResult>,
    pub total_chunk_pages: i64,
}

pub fn assemble_qdrant_filter(
    tag_set: Option<Vec<String>>,
    link: Option<Vec<String>>,
    time_range: Option<(String, String)>,
    filters: Option<serde_json::Value>,
    quote_words: Option<Vec<String>>,
    negated_words: Option<Vec<String>>,
    dataset_id: uuid::Uuid,
) -> Result<Filter, DefaultError> {
    let mut filter = Filter::default();

    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();

    filter
        .must
        .push(Condition::matches("dataset_id", dataset_id.to_string()));
    //TODO: fix this after new qdrant rust client gets released
    if !tag_set_inner.is_empty() {
        filter
            .must
            .push(Condition::matches("tag_set", tag_set_inner));
    }

    if !link_inner.is_empty() {
        filter.must.push(Condition::matches("link", link_inner));
    }

    if let Some(time_range) = time_range {
        if time_range.0 != "null" && time_range.1 != "null" {
            filter.must.push(Condition::range(
                "time_stamp",
                Range {
                    gt: None,
                    lt: None,
                    gte: Some(
                        time_range
                            .0
                            .clone()
                            .parse::<DateTimeUtc>()
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .0
                            .with_timezone(&chrono::Local)
                            .naive_local()
                            .timestamp() as f64,
                    ),
                    lte: Some(
                        time_range
                            .1
                            .clone()
                            .parse::<DateTimeUtc>()
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .0
                            .with_timezone(&chrono::Local)
                            .naive_local()
                            .timestamp() as f64,
                    ),
                },
            ));
        } else if time_range.1 == "null" {
            filter.must.push(Condition::range(
                "time_stamp",
                Range {
                    gt: None,
                    lt: None,
                    gte: Some(
                        time_range
                            .0
                            .clone()
                            .parse::<DateTimeUtc>()
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .0
                            .with_timezone(&chrono::Local)
                            .naive_local()
                            .timestamp() as f64,
                    ),
                    lte: None,
                },
            ));
        } else if time_range.0 == "null" {
            filter.must.push(Condition::range(
                "time_stamp",
                Range {
                    gt: None,
                    lt: None,
                    gte: None,
                    lte: Some(
                        time_range
                            .1
                            .clone()
                            .parse::<DateTimeUtc>()
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .0
                            .with_timezone(&chrono::Local)
                            .naive_local()
                            .timestamp() as f64,
                    ),
                },
            ));
        }
    }

    if let Some(serde_json::Value::Object(obj)) = &filters {
        for key in obj.keys() {
            let value = obj.get(key).expect("Value should exist");
            match value {
                serde_json::Value::Array(arr) => {
                    filter.must.push(Condition::matches(
                        &format!("metadata.{}", key),
                        arr.iter()
                            .map(|item| item.to_string())
                            .collect::<Vec<String>>(),
                    ));
                }
                _ => {
                    filter.must.push(Condition::matches(
                        &format!("metadata.{}", key),
                        value.to_string().replace('\"', ""),
                    ));
                }
            }
        }
    }

    //TODO: fix this after new qdrant rust client gets released
    if let Some(quote_words) = quote_words {
        for word in quote_words.iter() {
            filter
                .must
                .push(Condition::matches("card_html", word.clone() + " "));
        }
    }

    //TODO: fix this after new qdrant rust client gets released
    if let Some(negated_words) = negated_words {
        for word in negated_words.iter() {
            filter
                .must_not
                .push(Condition::matches("card_html", word.clone() + " "));
        }
    }

    Ok(filter)
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
) -> Result<SearchChunkQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };

    let filter = assemble_qdrant_filter(
        tag_set,
        link,
        time_range,
        filters,
        parsed_query.quote_words,
        parsed_query.negated_words,
        dataset_id,
    )?;

    let point_ids = if let Some(embedding_vector) = embedding_vector {
        search_semantic_qdrant_query(page, filter, embedding_vector).await?
    } else {
        search_full_text_qdrant_query(page, filter, parsed_query.query).await?
    };

    Ok(SearchChunkQueryResult {
        search_results: point_ids.clone(),
        //FIXME: dont have total results now
        total_chunk_pages: (point_ids.len() as f64 / 10.0).ceil() as i64,
    })
}

pub async fn global_unfiltered_top_match_query(
    embedding_vector: Vec<f32>,
    dataset_id: uuid::Uuid,
) -> Result<SearchResult, DefaultError> {
    let qdrant = get_qdrant_connection().await?;

    let qdrant_group = get_env!(
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
            collection_name: qdrant_group,
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
        // This only happens when there are no chunks in the database
        None => SearchResult {
            score: 0.0,
            point_id: uuid::Uuid::nil(),
        },
    };

    Ok(top_search_result)
}

#[allow(clippy::too_many_arguments)]
pub async fn search_semantic_chunk_groups_query(
    embedding_vector: Vec<f32>,
    page: u64,
    pool: web::Data<Pool>,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    filters: Option<serde_json::Value>,
    group_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    parsed_query: ParsedQuery,
) -> Result<SearchChunkQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().unwrap();

    let mut query = chunk_metadata_columns::chunk_metadata
        .left_outer_join(
            chunk_collisions_columns::chunk_collisions
                .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
        )
        .left_outer_join(
            chunk_group_bookmarks_columns::chunk_group_bookmarks.on(chunk_metadata_columns::id
                .eq(chunk_group_bookmarks_columns::chunk_metadata_id)
                .and(chunk_group_bookmarks_columns::group_id.eq(group_id))),
        )
        .select((
            chunk_metadata_columns::qdrant_point_id,
            chunk_collisions_columns::collision_qdrant_id.nullable(),
        ))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
        .filter(chunk_group_bookmarks_columns::group_id.eq(group_id))
        .distinct()
        .into_boxed();
    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();

    if let Some(tag) = tag_set_inner.first() {
        query = query.filter(chunk_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }
    for tag in tag_set_inner.iter().skip(1) {
        query = query.or_filter(chunk_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }

    if let Some(link_inner) = link_inner.first() {
        query = query.filter(chunk_metadata_columns::link.ilike(format!("%{}%", link_inner)));
    }
    for link_url in link_inner.iter().skip(1) {
        query = query.or_filter(chunk_metadata_columns::link.ilike(format!("%{}%", link_url)));
    }

    if let Some(serde_json::Value::Object(obj)) = &filters {
        for key in obj.keys() {
            if let Some(value) = obj.get(key) {
                match value {
                    serde_json::Value::Array(arr) => {
                        if let Some(first_val) = arr.first() {
                            if let Some(string_val) = first_val.as_str() {
                                query = query.filter(
                                    sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }

                        for item in arr.iter().skip(1) {
                            if let Some(string_val) = item.as_str() {
                                query = query.or_filter(
                                    sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }
                    }
                    serde_json::Value::String(string_val) => {
                        query = query.filter(
                            sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
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
            query = query.filter(chunk_metadata_columns::content.ilike(format!("%{}%", word)));
        }
    }

    if let Some(negated_words) = parsed_query.negated_words {
        for word in negated_words.iter() {
            query = query.filter(chunk_metadata_columns::content.not_ilike(format!("%{}%", word)));
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
        search_semantic_qdrant_query(page, filter, embedding_vector).await?;

    Ok(SearchChunkQueryResult {
        search_results: point_ids,
        total_chunk_pages: (filtered_option_ids.len() as f64 / 10.0).ceil() as i64,
    })
}

pub fn get_metadata_query(
    chunk_metadata: Vec<FullTextSearchResult>,
    mut conn: r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
) -> Result<Vec<ChunkMetadataWithFileData>, DefaultError> {
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::files::dsl as files_columns;
    use crate::data::schema::users::dsl as user_columns;

    let all_datas = chunk_metadata_columns::chunk_metadata
        .filter(
            chunk_metadata_columns::id.eq_any(
                chunk_metadata
                    .iter()
                    .map(|chunk| chunk.id)
                    .collect::<Vec<uuid::Uuid>>()
                    .as_slice(),
            ),
        )
        .left_outer_join(
            user_columns::users.on(chunk_metadata_columns::author_id.eq(user_columns::id)),
        )
        .left_outer_join(
            chunk_files_columns::chunk_files
                .on(chunk_metadata_columns::id.eq(chunk_files_columns::chunk_id)),
        )
        .left_outer_join(
            files_columns::files.on(chunk_files_columns::file_id.eq(files_columns::id)),
        )
        .left_outer_join(
            chunk_collisions_columns::chunk_collisions
                .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
        )
        .select((
            (
                chunk_files_columns::chunk_id,
                chunk_files_columns::file_id,
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
                user_columns::name,
            )
                .nullable(),
            (
                chunk_metadata_columns::id,
                chunk_collisions_columns::collision_qdrant_id.nullable(),
            ),
        ))
        .load::<(
            Option<ChunkFileWithName>,
            Option<User>,
            (uuid::Uuid, Option<uuid::Uuid>),
        )>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    #[allow(clippy::type_complexity)]
    let (file_ids, chunk_creators, chunk_collisions): (
        Vec<Option<ChunkFileWithName>>,
        Vec<Option<User>>,
        Vec<(uuid::Uuid, Option<uuid::Uuid>)>,
    ) = itertools::multiunzip(all_datas);

    let chunk_metadata_with_file_id: Vec<ChunkMetadataWithFileData> = chunk_metadata
        .into_iter()
        .map(|metadata| {
            let author = chunk_creators
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

            let chunk_with_file_name = file_ids
                .iter()
                .flatten()
                .find(|file| file.chunk_id == metadata.id);

            let qdrant_point_id = match metadata.qdrant_point_id {
                Some(id) => id,
                None => {
                    chunk_collisions
                                    .iter()
                                    .find(|collision| collision.0 == metadata.id) // Match chunk id
                                    .expect("Qdrant point id does not exist for root chunk or collision")
                                    .1
                                    .expect("Collision Qdrant point id must exist if there is no root qdrant point id")
                },
            };

            ChunkMetadataWithFileData {
                id: metadata.id,
                content: metadata.content,
                link: metadata.link,
                tag_set: metadata.tag_set,
                author,
                qdrant_point_id,
                created_at: metadata.created_at,
                updated_at: metadata.updated_at,
                chunk_html: metadata.chunk_html,
                file_id: chunk_with_file_name.map(|file| file.file_id),
                file_name: chunk_with_file_name.map(|file| file.file_name.to_string()),
                metadata: metadata.metadata,
                tracking_id: metadata.tracking_id,
                time_stamp: metadata.time_stamp,
                weight: metadata.weight
            }
        })
        .collect();
    Ok(chunk_metadata_with_file_id)
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct FullTextDocIds {
    pub doc_ids: Option<uuid::Uuid>,
    pub total_count: i64,
}

#[allow(clippy::too_many_arguments)]
pub async fn search_full_text_group_query(
    user_query: String,
    page: u64,
    pool: web::Data<Pool>,
    filters: Option<serde_json::Value>,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    group_id: uuid::Uuid,
    parsed_query: ParsedQuery,
    dataset_uuid: uuid::Uuid,
) -> Result<SearchChunkQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let second_join = diesel::alias!(schema::chunk_metadata as second_join);

    let mut conn = pool.get().unwrap();
    // SELECT
    //     chunk_metadata.qdrant_point_id,
    //     second_join.qdrant_point_id
    // FROM
    //     chunk_metadata
    // LEFT OUTER JOIN chunk_collisions ON
    //     chunk_metadata.id = chunk_collisions.chunk_id
    //     AND chunk_metadata.private = false
    // LEFT OUTER JOIN chunk_metadata AS second_join ON
    //     second_join.qdrant_point_id = chunk_collisions.collision_qdrant_id
    //     AND second_join.private = true
    // WHERE
    //     chunk_metadata.private = false
    //     and (second_join.qdrant_point_id notnull or chunk_metadata.qdrant_point_id notnull);
    let mut query = chunk_metadata_columns::chunk_metadata
        .left_outer_join(
            chunk_collisions_columns::chunk_collisions
                .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
        )
        .left_outer_join(
            second_join.on(second_join
                .field(schema::chunk_metadata::qdrant_point_id)
                .eq(chunk_collisions_columns::collision_qdrant_id)),
        )
        .left_outer_join(
            chunk_group_bookmarks_columns::chunk_group_bookmarks.on(chunk_metadata_columns::id
                .eq(chunk_group_bookmarks_columns::chunk_metadata_id)
                .and(chunk_group_bookmarks_columns::group_id.eq(group_id))),
        )
        .filter(chunk_group_bookmarks_columns::group_id.eq(group_id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid))
        .select((
            chunk_metadata_columns::qdrant_point_id,
            second_join
                .field(schema::chunk_metadata::qdrant_point_id)
                .nullable(),
        ))
        .distinct_on((
            chunk_metadata_columns::qdrant_point_id,
            second_join
                .field(schema::chunk_metadata::qdrant_point_id)
                .nullable(),
        ))
        .into_boxed();

    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();

    if let Some(tag) = tag_set_inner.first() {
        query = query.filter(chunk_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }
    for tag in tag_set_inner.iter().skip(1) {
        query = query.or_filter(chunk_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }

    if let Some(link_inner) = link_inner.first() {
        query = query.filter(chunk_metadata_columns::link.ilike(format!("%{}%", link_inner)));
    }
    for link_url in link_inner.iter().skip(1) {
        query = query.or_filter(chunk_metadata_columns::link.ilike(format!("%{}%", link_url)));
    }

    if let Some(serde_json::Value::Object(obj)) = &filters {
        for key in obj.keys() {
            if let Some(value) = obj.get(key) {
                match value {
                    serde_json::Value::Array(arr) => {
                        if let Some(first_val) = arr.first() {
                            if let Some(string_val) = first_val.as_str() {
                                query = query.filter(
                                    sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }

                        for item in arr.iter().skip(1) {
                            if let Some(string_val) = item.as_str() {
                                query = query.or_filter(
                                    sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }
                    }
                    serde_json::Value::String(string_val) => {
                        query = query.filter(
                            sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
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
            query = query.filter(chunk_metadata_columns::content.ilike(format!("%{}%", word)));
        }
    }

    if let Some(negated_words) = parsed_query.negated_words {
        for word in negated_words.iter() {
            query = query.filter(chunk_metadata_columns::content.not_ilike(format!("%{}%", word)));
        }
    }

    query = query.order((
        chunk_metadata_columns::qdrant_point_id,
        second_join.field(schema::chunk_metadata::qdrant_point_id),
    ));

    let matching_qdrant_point_ids: Vec<(Option<uuid::Uuid>, Option<uuid::Uuid>)> =
        query.load(&mut conn).map_err(|_| DefaultError {
            message: "Failed to load full-text searched chunks",
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

    let point_ids = search_full_text_qdrant_query(page, filter, user_query).await;

    Ok(SearchChunkQueryResult {
        search_results: point_ids?,
        total_chunk_pages: (matching_qdrant_point_ids.len() as f64 / 10.0).ceil() as i64,
    })
}

pub async fn retrieve_chunks_from_point_ids_without_collsions(
    search_chunk_query_results: SearchChunkQueryResult,
    data: &web::Json<SearchChunkData>,
    dataset_config: &ServerDatasetConfiguration,
    pool: web::Data<Pool>,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    let point_ids = search_chunk_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let metadata_chunks = get_metadata_from_point_ids(point_ids, pool)
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_chunks: Vec<ScoreChunkDTO> = search_chunk_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut chunk: ChunkMetadataWithFileData = match metadata_chunks
                .iter()
                .find(|metadata_chunk| metadata_chunk.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_chunk) => metadata_chunk.clone(),
                None => ChunkMetadataWithFileData {
                    id: uuid::Uuid::default(),
                    author: None,
                    qdrant_point_id: uuid::Uuid::default(),
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    file_id: None,
                    file_name: None,
                    content: "".to_string(),
                    chunk_html: Some("".to_string()),
                    link: Some("".to_string()),
                    tag_set: Some("".to_string()),
                    metadata: None,
                    tracking_id: None,
                    time_stamp: None,
                    weight: 1.0,
                },
            };
            if dataset_config.HIGHLIGHT_ENABLED.unwrap_or(true) {
                chunk = find_relevant_sentence(
                    chunk.clone(),
                    data.query.clone(),
                    dataset_config
                        .HIGHLIGHT_SPLIT_DELIMITERS
                        .clone()
                        .unwrap_or(vec![
                            ".".to_string(),
                            "!".to_string(),
                            "?".to_string(),
                            "\n".to_string(),
                            "\t".to_string(),
                            ",".to_string(),
                        ]),
                )
                .unwrap_or(chunk);
            }

            ScoreChunkDTO {
                metadata: vec![chunk],
                score: search_result.score.into(),
            }
        })
        .collect();

    Ok(SearchChunkQueryResponseBody {
        score_chunks,
        total_chunk_pages: search_chunk_query_results.total_chunk_pages,
    })
}

/// Retrieve chunks from point ids, DOES NOT GUARD AGAINST DATASET ACCESS PERMISSIONS
pub async fn retrieve_chunks_from_point_ids(
    search_chunk_query_results: SearchChunkQueryResult,
    data: &web::Json<SearchChunkData>,
    dataset_config: &ServerDatasetConfiguration,
    pool: web::Data<Pool>,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    let point_ids = search_chunk_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let (metadata_chunks, collided_chunks) = get_metadata_and_collided_chunks_from_point_ids_query(
        point_ids,
        data.get_collisions.unwrap_or(false),
        pool,
    )
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_chunks: Vec<ScoreChunkDTO> = search_chunk_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut chunk: ChunkMetadataWithFileData = match metadata_chunks
                .iter()
                .find(|metadata_chunk| metadata_chunk.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_chunk) => metadata_chunk.clone(),
                None => ChunkMetadataWithFileData {
                    id: uuid::Uuid::default(),
                    author: None,
                    qdrant_point_id: uuid::Uuid::default(),
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    file_id: None,
                    file_name: None,
                    content: "".to_string(),
                    chunk_html: Some("".to_string()),
                    link: Some("".to_string()),
                    tag_set: Some("".to_string()),
                    metadata: None,
                    tracking_id: None,
                    time_stamp: None,
                    weight: 1.0,
                },
            };

            if dataset_config.HIGHLIGHT_ENABLED.unwrap_or(true) {
                chunk = find_relevant_sentence(
                    chunk.clone(),
                    data.query.clone(),
                    dataset_config
                        .HIGHLIGHT_SPLIT_DELIMITERS
                        .clone()
                        .unwrap_or(vec![
                            ".".to_string(),
                            "!".to_string(),
                            "?".to_string(),
                            "\n".to_string(),
                            "\t".to_string(),
                            ",".to_string(),
                        ]),
                )
                .unwrap_or(chunk);
            }

            let mut collided_chunks: Vec<ChunkMetadataWithFileData> = collided_chunks
                .iter()
                .filter(|chunk| chunk.qdrant_id == search_result.point_id)
                .map(|chunk| chunk.metadata.clone())
                .collect();

            collided_chunks.insert(0, chunk);

            ScoreChunkDTO {
                metadata: collided_chunks,
                score: search_result.score.into(),
            }
        })
        .collect();

    Ok(SearchChunkQueryResponseBody {
        score_chunks,
        total_chunk_pages: search_chunk_query_results.total_chunk_pages,
    })
}

pub fn rerank_chunks(chunks: Vec<ScoreChunkDTO>, date_bias: Option<bool>) -> Vec<ScoreChunkDTO> {
    let mut reranked_chunks = Vec::new();
    chunks.into_iter().for_each(|mut chunk| {
        if chunk.metadata[0].weight == 0.0 {
            chunk.metadata[0].weight = 1.0;
        }
        chunk.score *= chunk.metadata[0].weight;
        reranked_chunks.push(chunk);
    });

    if date_bias.is_some() && date_bias.unwrap() {
        reranked_chunks.iter_mut().for_each(|chunk| {
            if let Some(time_stamp) = chunk.metadata[0].time_stamp {
                let time_stamp = time_stamp.timestamp();
                let now = chrono::Utc::now().timestamp();
                let time_diff = now - time_stamp;
                let time_diff = time_diff as f32 / 60.0 / 60.0 / 24.0;
                chunk.score *= E.powf(-0.1 * time_diff) as f64;
            }
        });
    }

    reranked_chunks.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    reranked_chunks
}

pub async fn search_semantic_chunks(
    data: web::Json<SearchChunkData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());
    let embedding_vector = create_embedding(&data.query, dataset_config.clone()).await?;

    let search_chunk_query_results = retrieve_qdrant_points_query(
        Some(embedding_vector),
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        parsed_query,
        dataset.id,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut result_chunks = retrieve_chunks_from_point_ids(
        search_chunk_query_results,
        &data,
        &dataset_config,
        pool.clone(),
    )
    .await?;

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);

    Ok(result_chunks)
}

pub async fn search_full_text_chunks(
    data: web::Json<SearchChunkData>,
    mut parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());
    parsed_query.query = parsed_query
        .query
        .split_whitespace()
        .join(" AND ")
        .replace('\"', "");

    let search_chunk_query_results = retrieve_qdrant_points_query(
        None,
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        parsed_query,
        dataset.id,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut result_chunks =
        retrieve_chunks_from_point_ids(search_chunk_query_results, &data, &dataset_config, pool)
            .await?;

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);

    Ok(result_chunks)
}

fn reciprocal_rank_fusion(
    semantic_results: Vec<ScoreChunkDTO>,
    full_text_results: Vec<ScoreChunkDTO>,
    weights: Option<(f64, f64)>,
) -> Vec<ScoreChunkDTO> {
    let mut fused_ranking: Vec<ScoreChunkDTO> = Vec::new();
    let weights = weights.unwrap_or((1.0, 1.0));
    // Iterate through the union of the two result sets
    for mut document in full_text_results
        .clone()
        .into_iter()
        .chain(semantic_results.clone().into_iter())
        .unique_by(|chunk| chunk.metadata[0].id)
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
pub async fn search_hybrid_chunks(
    data: web::Json<SearchChunkData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());
    let embedding_vector = create_embedding(&data.query, dataset_config.clone()).await?;
    let pool1 = pool.clone();

    let search_chunk_query_results = retrieve_qdrant_points_query(
        Some(embedding_vector),
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        parsed_query.clone(),
        dataset.id,
    );

    let full_text_handler_results =
        search_full_text_chunks(web::Json(data.clone()), parsed_query, page, pool, dataset);

    let (search_chunk_query_results, full_text_handler_results) =
        futures::join!(search_chunk_query_results, full_text_handler_results);

    let search_chunk_query_results =
        search_chunk_query_results.map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let full_text_handler_results =
        full_text_handler_results.map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let point_ids = search_chunk_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let (metadata_chunks, collided_chunks) = get_metadata_and_collided_chunks_from_point_ids_query(
        point_ids,
        data.get_collisions.unwrap_or(false),
        pool1,
    )
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let semantic_score_chunks: Vec<ScoreChunkDTO> = search_chunk_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut chunk: ChunkMetadataWithFileData = match metadata_chunks
                .iter()
                .find(|metadata_chunk| metadata_chunk.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_chunk) => metadata_chunk.clone(),
                None => ChunkMetadataWithFileData {
                    id: uuid::Uuid::default(),
                    author: None,
                    qdrant_point_id: uuid::Uuid::default(),
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    file_id: None,
                    file_name: None,
                    content: "".to_string(),
                    chunk_html: Some("".to_string()),
                    link: Some("".to_string()),
                    tag_set: Some("".to_string()),
                    metadata: None,
                    tracking_id: None,
                    time_stamp: None,
                    weight: 1.0,
                },
            };

            if dataset_config.HIGHLIGHT_ENABLED.unwrap_or(true) {
                chunk = find_relevant_sentence(
                    chunk.clone(),
                    data.query.clone(),
                    dataset_config
                        .HIGHLIGHT_SPLIT_DELIMITERS
                        .clone()
                        .unwrap_or(vec![
                            ".".to_string(),
                            "!".to_string(),
                            "?".to_string(),
                            "\n".to_string(),
                            "\t".to_string(),
                            ",".to_string(),
                        ]),
                )
                .unwrap_or(chunk);
            }

            let mut collided_chunks: Vec<ChunkMetadataWithFileData> = collided_chunks
                .iter()
                .filter(|chunk| chunk.qdrant_id == search_result.point_id)
                .map(|chunk| chunk.metadata.clone())
                .collect();

            collided_chunks.insert(0, chunk);

            ScoreChunkDTO {
                metadata: collided_chunks,
                score: search_result.score as f64 * 0.5,
            }
        })
        .collect();

    let mut result_chunks = if data.cross_encoder.unwrap_or(false) {
        let combined_results = semantic_score_chunks
            .into_iter()
            .chain(full_text_handler_results.score_chunks.into_iter())
            .unique_by(|score_chunk| score_chunk.metadata[0].id)
            .collect::<Vec<ScoreChunkDTO>>();
        SearchChunkQueryResponseBody {
            score_chunks: cross_encoder(data.query.clone(), combined_results).await?,
            total_chunk_pages: search_chunk_query_results.total_chunk_pages,
        }
    } else if let Some(weights) = data.weights {
        if weights.0 == 1.0 {
            SearchChunkQueryResponseBody {
                score_chunks: semantic_score_chunks,
                total_chunk_pages: search_chunk_query_results.total_chunk_pages,
            }
        } else if weights.1 == 1.0 {
            SearchChunkQueryResponseBody {
                score_chunks: full_text_handler_results.score_chunks,
                total_chunk_pages: full_text_handler_results.total_chunk_pages,
            }
        } else {
            SearchChunkQueryResponseBody {
                score_chunks: reciprocal_rank_fusion(
                    semantic_score_chunks,
                    full_text_handler_results.score_chunks,
                    data.weights,
                ),
                total_chunk_pages: search_chunk_query_results.total_chunk_pages,
            }
        }
    } else {
        SearchChunkQueryResponseBody {
            score_chunks: reciprocal_rank_fusion(
                semantic_score_chunks,
                full_text_handler_results.score_chunks,
                data.weights,
            ),
            total_chunk_pages: search_chunk_query_results.total_chunk_pages,
        }
    };

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);

    Ok(result_chunks)
}

#[allow(clippy::too_many_arguments)]
pub async fn search_semantic_groups(
    data: web::Json<SearchGroupsData>,
    parsed_query: ParsedQuery,
    group: ChunkGroup,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchGroupsResult, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());
    let embedding_vector: Vec<f32> = create_embedding(&data.query, dataset_config.clone()).await?;

    let search_semantic_chunk_query_results = search_semantic_chunk_groups_query(
        embedding_vector,
        page,
        pool.clone(),
        data.link.clone(),
        data.tag_set.clone(),
        data.filters.clone(),
        data.group_id,
        dataset.id,
        parsed_query,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut result_chunks = retrieve_chunks_from_point_ids_without_collsions(
        search_semantic_chunk_query_results,
        &web::Json(data.clone().into()),
        &dataset_config,
        pool.clone(),
    )
    .await?;

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);

    Ok(SearchGroupsResult {
        bookmarks: result_chunks.score_chunks,
        group,
        total_pages: result_chunks.total_chunk_pages,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn search_full_text_groups(
    data: web::Json<SearchGroupsData>,
    parsed_query: ParsedQuery,
    group: ChunkGroup,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchGroupsResult, actix_web::Error> {
    let data_inner = data.clone();
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());
    let search_chunk_query_results = search_full_text_group_query(
        data_inner.query.clone(),
        page,
        pool.clone(),
        data_inner.filters.clone(),
        data_inner.link.clone(),
        data_inner.tag_set.clone(),
        data_inner.group_id,
        parsed_query,
        dataset.id,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut result_chunks = retrieve_chunks_from_point_ids_without_collsions(
        search_chunk_query_results,
        &web::Json(data.clone().into()),
        &dataset_config,
        pool.clone(),
    )
    .await?;

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);

    Ok(SearchGroupsResult {
        bookmarks: result_chunks.score_chunks,
        group,
        total_pages: result_chunks.total_chunk_pages,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn search_hybrid_groups(
    data: web::Json<SearchGroupsData>,
    parsed_query: ParsedQuery,
    group: ChunkGroup,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchGroupsResult, actix_web::Error> {
    let data_inner = data.clone();
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

    let embedding_vector = create_embedding(&data.query, dataset_config.clone()).await?;

    let semantic_future = search_semantic_chunk_groups_query(
        embedding_vector,
        page,
        pool.clone(),
        data.link.clone(),
        data.tag_set.clone(),
        data.filters.clone(),
        data.group_id,
        dataset.id,
        parsed_query.clone(),
    );

    let full_text_future = search_full_text_group_query(
        data_inner.query.clone(),
        page,
        pool.clone(),
        data_inner.filters.clone(),
        data_inner.link.clone(),
        data_inner.tag_set.clone(),
        data_inner.group_id,
        parsed_query.clone(),
        dataset.id,
    );

    let (semantic_results, full_text_results) = futures::join!(semantic_future, full_text_future);

    let semantic_results =
        semantic_results.map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let full_text_results =
        full_text_results.map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let combined_results = semantic_results
        .clone()
        .search_results
        .into_iter()
        .chain(full_text_results.clone().search_results.into_iter())
        .unique_by(|chunk| chunk.point_id)
        .collect::<Vec<SearchResult>>();

    let combined_search_chunk_query_results = SearchChunkQueryResult {
        search_results: combined_results,
        total_chunk_pages: semantic_results.total_chunk_pages,
    };

    let combined_result_chunks = retrieve_chunks_from_point_ids_without_collsions(
        combined_search_chunk_query_results,
        &web::Json(data.clone().into()),
        &dataset_config,
        pool.clone(),
    )
    .await?;

    let semantic_chunk_dtos = semantic_results
        .search_results
        .iter()
        .map(|search_result| {
            combined_result_chunks
                .score_chunks
                .iter()
                .find(|metadata_chunk| {
                    metadata_chunk
                        .metadata
                        .first()
                        .expect("Must be at least one metadata in result")
                        .qdrant_point_id
                        == search_result.point_id
                })
                .expect("Was not able to find semantic chunk in combined result chunks")
                .clone()
        })
        .collect::<Vec<ScoreChunkDTO>>();
    let full_text_chunk_dtos = full_text_results
        .search_results
        .iter()
        .map(|search_result| {
            combined_result_chunks
                .score_chunks
                .iter()
                .find(|metadata_chunk| {
                    metadata_chunk
                        .metadata
                        .first()
                        .expect("Must be at least one metadata in result")
                        .qdrant_point_id
                        == search_result.point_id
                })
                .expect("Was not able to find full-text chunk in combined result chunks")
                .clone()
        })
        .collect::<Vec<ScoreChunkDTO>>();

    let mut result_chunks = if data.cross_encoder.unwrap_or(false) {
        let combined_results = semantic_chunk_dtos
            .into_iter()
            .chain(full_text_chunk_dtos.into_iter())
            .unique_by(|score_chunk| score_chunk.metadata[0].id)
            .collect::<Vec<ScoreChunkDTO>>();
        SearchChunkQueryResponseBody {
            score_chunks: cross_encoder(data.query.clone(), combined_results).await?,
            total_chunk_pages: semantic_results.total_chunk_pages,
        }
    } else if let Some(weights) = data.weights {
        if weights.0 == 1.0 {
            SearchChunkQueryResponseBody {
                score_chunks: semantic_chunk_dtos,
                total_chunk_pages: semantic_results.total_chunk_pages,
            }
        } else if weights.1 == 1.0 {
            SearchChunkQueryResponseBody {
                score_chunks: full_text_chunk_dtos,
                total_chunk_pages: full_text_results.total_chunk_pages,
            }
        } else {
            SearchChunkQueryResponseBody {
                score_chunks: reciprocal_rank_fusion(
                    semantic_chunk_dtos,
                    full_text_chunk_dtos,
                    data.weights,
                ),
                total_chunk_pages: semantic_results.total_chunk_pages,
            }
        }
    } else {
        SearchChunkQueryResponseBody {
            score_chunks: reciprocal_rank_fusion(
                semantic_chunk_dtos,
                full_text_chunk_dtos,
                data.weights,
            ),
            total_chunk_pages: semantic_results.total_chunk_pages,
        }
    };

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);

    Ok(SearchGroupsResult {
        bookmarks: combined_result_chunks.score_chunks,
        group,
        total_pages: combined_result_chunks.total_chunk_pages,
    })
}
