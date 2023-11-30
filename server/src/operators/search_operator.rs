use crate::data::models::{
    CardCollection, CardFileWithName, CardMetadataWithVotesWithScore, CardVote,
    FullTextSearchResult, User, UserDTO,
};
use crate::data::schema::{self};
use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use crate::errors::ServiceError;
use crate::handlers::card_handler::{
    ParsedQuery, ScoreCardDTO, SearchCardData, SearchCardQueryResponseBody, SearchCollectionsData,
    SearchCollectionsResult,
};
use crate::operators::qdrant_operator::get_qdrant_connection;
use crate::operators::qdrant_operator::search_qdrant_query;
use crate::CrossEncoder;
use crate::{data::models::Pool, errors::DefaultError};
use actix_web::web;
use chrono::NaiveDateTime;
use diesel::dsl::sql;
use diesel::sql_types::Int8;
use diesel::sql_types::Nullable;
use diesel::sql_types::Text;
use diesel::sql_types::{Bool, Double};
use diesel::{
    BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgTextExpressionMethods,
};

use crate::AppMutexStore;
use futures_util::TryFutureExt;
use itertools::Itertools;
use pyo3::types::PyDict;
use pyo3::{IntoPy, Python};
use qdrant_client::qdrant::condition::ConditionOneOf::HasId;
use qdrant_client::qdrant::{
    point_id::PointIdOptions, Condition, Filter, HasIdCondition, PointId, Range, SearchPoints,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;


use super::card_operator::{
    find_relevant_sentence, get_collided_cards_query,
    get_metadata_and_collided_cards_from_point_ids_query, get_metadata_from_point_ids,
};
use super::qdrant_operator::create_embedding;

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
    embedding_vector: Vec<f32>,
    page: u64,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    time_range: Option<(String, String)>,
    filters: Option<serde_json::Value>,
    current_user_id: Option<uuid::Uuid>,
    parsed_query: ParsedQuery,
) -> Result<SearchCardQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };

    let mut filter = Filter::default();
    filter.should.push(Condition::is_empty("private"));
    filter.should.push(Condition::is_null("private"));
    filter.should.push(Condition::matches("private", false));
    filter.should.push(Condition::matches(
        "authors",
        current_user_id.unwrap_or_default().to_string(),
    ));

    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();

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
                    gt: Some(
                        NaiveDateTime::parse_from_str(&time_range.0, "%Y-%m-%d %H:%M:%S")
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .timestamp() as f64,
                    ),
                    lt: Some(
                        NaiveDateTime::parse_from_str(&time_range.1, "%Y-%m-%d %H:%M:%S")
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .timestamp() as f64,
                    ),
                    gte: None,
                    lte: None,
                },
            ));
        } else if time_range.1 == "null" {
            filter.must.push(Condition::range(
                "time_stamp",
                Range {
                    gt: Some(
                        NaiveDateTime::parse_from_str(&time_range.0, "%Y-%m-%d %H:%M:%S")
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .timestamp() as f64,
                    ),
                    lt: None,
                    gte: None,
                    lte: None,
                },
            ));
        } else if time_range.0 == "null" {
            filter.must.push(Condition::range(
                "time_stamp",
                Range {
                    gt: None,
                    lt: Some(
                        NaiveDateTime::parse_from_str(&time_range.1, "%Y-%m-%d %H:%M:%S")
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .timestamp() as f64,
                    ),
                    gte: None,
                    lte: None,
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

    if let Some(quote_words) = parsed_query.quote_words {
        for word in quote_words.iter() {
            filter
                .must
                .push(Condition::matches("card_html", word.clone() + " "));
        }
    }

    if let Some(negated_words) = parsed_query.negated_words {
        for word in negated_words.iter() {
            filter
                .must_not
                .push(Condition::matches("card_html", word.clone()));
        }
    }

    let point_ids = search_qdrant_query(page, filter, embedding_vector.clone()).await?;

    Ok(SearchCardQueryResult {
        search_results: point_ids,
        total_card_pages: 100,
    })
}

pub async fn global_unfiltered_top_match_query(
    embedding_vector: Vec<f32>,
) -> Result<SearchResult, DefaultError> {
    let qdrant = get_qdrant_connection().await?;

    let qdrant_collection = std::env::var("QDRANT_COLLECTION").unwrap_or("debate_cards".to_owned());
    let data = qdrant
        .search_points(&SearchPoints {
            collection_name: qdrant_collection,
            vector: embedding_vector,
            limit: 1,
            with_payload: None,
            ..Default::default()
        })
        .await
        .map_err(|e| {
            log::error!("Failed to search points on Qdrant {:?}", e);
            DefaultError {
                message: "Failed to search points on Qdrant",
            }
        })?;

    let top_search_result: SearchResult = match data.result.get(0) {
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
    user_id: Option<uuid::Uuid>,
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
        .filter(
            card_metadata_columns::private
                .eq(false)
                .or(card_metadata_columns::author_id.eq(user_id.unwrap_or(uuid::Uuid::nil()))),
        )
        .filter(card_collection_bookmarks_columns::collection_id.eq(collection_id))
        .distinct()
        .into_boxed();
    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();

    if let Some(tag) = tag_set_inner.get(0) {
        query = query.filter(card_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }
    for tag in tag_set_inner.iter().skip(1) {
        query = query.or_filter(card_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }

    if let Some(link_inner) = link_inner.get(0) {
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
                        if let Some(first_val) = arr.get(0) {
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

    let point_ids: Vec<SearchResult> = search_qdrant_query(page, filter, embedding_vector).await?;

    Ok(SearchCardQueryResult {
        search_results: point_ids,
        total_card_pages: (filtered_option_ids.len() as f64 / 10.0).ceil() as i64,
    })
}

pub fn get_metadata_query(
    card_metadata: Vec<FullTextSearchResult>,
    current_user_id: Option<uuid::Uuid>,
    mut conn: r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
) -> Result<Vec<CardMetadataWithVotesWithScore>, DefaultError> {
    use crate::data::schema::card_collisions::dsl as card_collisions_columns;
    use crate::data::schema::card_files::dsl as card_files_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    use crate::data::schema::card_votes::dsl as card_votes_columns;
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
            card_votes_columns::card_votes
                .on(card_metadata_columns::id.eq(card_votes_columns::card_metadata_id)),
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
                card_votes_columns::id,
                card_votes_columns::voted_user_id,
                card_votes_columns::card_metadata_id,
                card_votes_columns::vote,
                card_votes_columns::created_at,
                card_votes_columns::updated_at,
                card_votes_columns::deleted,
            )
                .nullable(),
            (
                user_columns::id,
                user_columns::email,
                user_columns::hash,
                user_columns::created_at,
                user_columns::updated_at,
                user_columns::username,
                user_columns::website,
                user_columns::visible_email,
                user_columns::api_key_hash,
            )
                .nullable(),
            (
                card_metadata_columns::id,
                card_collisions_columns::collision_qdrant_id.nullable(),
            ),
        ))
        .load::<(
            Option<CardFileWithName>,
            Option<CardVote>,
            Option<User>,
            (uuid::Uuid, Option<uuid::Uuid>),
        )>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    #[allow(clippy::type_complexity)]
    let (file_ids, card_votes, card_creators, card_collisions): (
        Vec<Option<CardFileWithName>>,
        Vec<Option<CardVote>>,
        Vec<Option<User>>,
        Vec<(uuid::Uuid, Option<uuid::Uuid>)>,
    ) = itertools::multiunzip(all_datas);

    let card_metadata_with_upvotes_and_file_id: Vec<CardMetadataWithVotesWithScore> = card_metadata
        .into_iter()
        .map(|metadata| {
            let votes = card_votes
                .iter()
                .flatten()
                .filter(|upvote| upvote.card_metadata_id == metadata.id)
                .collect::<Vec<&CardVote>>();
            let total_upvotes = votes.iter().filter(|upvote| upvote.vote).count() as i64;
            let total_downvotes = votes.iter().filter(|upvote| !upvote.vote).count() as i64;
            let vote_by_current_user = match current_user_id {
                Some(user_id) => votes
                    .iter()
                    .find(|upvote| upvote.voted_user_id == user_id)
                    .map(|upvote| upvote.vote),
                None => None,
            };

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

            CardMetadataWithVotesWithScore {
                id: metadata.id,
                content: metadata.content,
                link: metadata.link,
                tag_set: metadata.tag_set,
                author,
                qdrant_point_id,
                total_upvotes,
                total_downvotes,
                vote_by_current_user,
                created_at: metadata.created_at,
                updated_at: metadata.updated_at,
                private: metadata.private,
                score: metadata.score,
                card_html: metadata.card_html,
                file_id: card_with_file_name.map(|file| file.file_id),
                file_name: card_with_file_name.map(|file| file.file_name.to_string()),
                metadata: metadata.metadata,
                tracking_id: metadata.tracking_id,
                time_stamp: metadata.time_stamp,
            }
        })
        .collect();
    Ok(card_metadata_with_upvotes_and_file_id)
}

#[derive(Serialize, Deserialize)]
pub struct FullTextSearchCardQueryResult {
    pub search_results: Vec<CardMetadataWithVotesWithScore>,
    pub total_card_pages: i64,
}

#[allow(clippy::too_many_arguments)]
pub async fn search_full_text_card_query(
    user_query: String,
    page: u64,
    pool: web::Data<Pool>,
    current_user_id: Option<uuid::Uuid>,
    filters: Option<serde_json::Value>,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    time_range: Option<(String, String)>,
    parsed_query: ParsedQuery,
) -> Result<FullTextSearchCardQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };
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
            card_collisions_columns::card_collisions.on(card_metadata_columns::id
                .eq(card_collisions_columns::card_id)
                .and(card_metadata_columns::private.eq(false))),
        )
        .left_outer_join(
            second_join.on(second_join
                .field(schema::card_metadata::qdrant_point_id)
                .eq(card_collisions_columns::collision_qdrant_id)
                .and(second_join.field(schema::card_metadata::private).eq(true))),
        )
        .filter(
            card_metadata_columns::private
                .eq(false)
                .or(card_metadata_columns::author_id
                    .eq(current_user_id.unwrap_or(uuid::Uuid::nil())))
                .and(
                    second_join
                        .field(schema::card_metadata::qdrant_point_id)
                        .is_not_null()
                        .or(card_metadata_columns::qdrant_point_id.is_not_null()),
                ),
        )
        .select((
            (
                card_metadata_columns::id,
                card_metadata_columns::content,
                card_metadata_columns::link,
                card_metadata_columns::author_id,
                card_metadata_columns::qdrant_point_id,
                card_metadata_columns::created_at,
                card_metadata_columns::updated_at,
                card_metadata_columns::tag_set,
                card_metadata_columns::card_html,
                card_metadata_columns::private,
                card_metadata_columns::metadata,
                card_metadata_columns::tracking_id,
                card_metadata_columns::time_stamp,
                sql::<Nullable<Double>>(
                    "paradedb.rank_bm25(card_metadata.ctid)::double precision as rank",
                ),
                sql::<Int8>("count(*) OVER() AS full_count"),
            ),
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

    //escape special characters
    query = query.filter(sql::<Bool>(
        format!("card_metadata @@@ '{}'", user_query).as_str(),
    ));
    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();
    if !tag_set_inner.is_empty() {
        query = query.filter(card_metadata_columns::tag_set.ilike(format!(
            "%{}%",
            tag_set_inner.get(0).unwrap_or(&String::new())
        )));
    }

    for tag in tag_set_inner.iter().skip(1) {
        query = query.or_filter(card_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }

    if !link_inner.is_empty() {
        query = query.filter(
            card_metadata_columns::link
                .ilike(format!("%{}%", link_inner.get(0).unwrap_or(&String::new()))),
        );
    }
    for link_url in link_inner.iter().skip(1) {
        query = query.or_filter(card_metadata_columns::link.ilike(format!("%{}%", link_url)));
    }

    if let Some(time_range) = time_range {
        if time_range.0 != "null" && time_range.1 != "null" {
            query = query.filter(
                card_metadata_columns::time_stamp
                    .gt(
                        NaiveDateTime::parse_from_str(&time_range.0, "%Y-%m-%d %H:%M:%S").map_err(
                            |_| DefaultError {
                                message: "Failed to parse time range",
                            },
                        )?,
                    )
                    .and(card_metadata_columns::time_stamp.lt(
                        NaiveDateTime::parse_from_str(&time_range.1, "%Y-%m-%d %H:%M:%S").map_err(
                            |_| DefaultError {
                                message: "Failed to parse time range",
                            },
                        )?,
                    )),
            );
        } else if time_range.0 != "null" {
            query = query.filter(card_metadata_columns::time_stamp.gt(
                NaiveDateTime::parse_from_str(&time_range.0, "%Y-%m-%d %H:%M:%S").map_err(
                    |_| DefaultError {
                        message: "Failed to parse time range",
                    },
                )?,
            ));
        } else if time_range.1 != "null" {
            query = query.filter(card_metadata_columns::time_stamp.lt(
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
                            .ilike(format!("%{}%", arr.get(0).unwrap().as_str().unwrap_or(""))),
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

    query = query.order((
        card_metadata_columns::qdrant_point_id,
        second_join.field(schema::card_metadata::qdrant_point_id),
        sql::<Text>("rank DESC"),
    ));

    query = query
        .limit(10)
        .offset(((page - 1) * 10).try_into().unwrap_or(0));

    let searched_cards: Vec<(FullTextSearchResult, Option<uuid::Uuid>)> =
        query.load(&mut conn).map_err(|_| DefaultError {
            message: "Failed to load full-text searched cards",
        })?;

    let card_metadata_with_upvotes_and_files = get_metadata_query(
        searched_cards
            .iter()
            .map(|card| card.0.clone())
            .collect::<Vec<FullTextSearchResult>>(),
        current_user_id,
        conn,
    )
    .map_err(|_| DefaultError {
        message: "Failed to load searched cards",
    })?;

    let total_count = if searched_cards.is_empty() {
        0
    } else {
        (searched_cards
            .get(0)
            .expect("searched_cards should have a len of at least 1")
            .0
            .count as f64
            / 10.0)
            .ceil() as i64
    };

    Ok(FullTextSearchCardQueryResult {
        search_results: card_metadata_with_upvotes_and_files,
        total_card_pages: total_count,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn search_full_text_collection_query(
    user_query: String,
    page: u64,
    pool: web::Data<Pool>,
    current_user_id: Option<uuid::Uuid>,
    filters: Option<serde_json::Value>,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    collection_id: uuid::Uuid,
    parsed_query: ParsedQuery,
) -> Result<FullTextSearchCardQueryResult, DefaultError> {
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
            card_collisions_columns::card_collisions.on(card_metadata_columns::id
                .eq(card_collisions_columns::card_id)
                .and(card_metadata_columns::private.eq(false))),
        )
        .left_outer_join(
            second_join.on(second_join
                .field(schema::card_metadata::qdrant_point_id)
                .eq(card_collisions_columns::collision_qdrant_id)
                .and(second_join.field(schema::card_metadata::private).eq(true))),
        )
        .left_outer_join(
            card_collection_bookmarks_columns::card_collection_bookmarks.on(
                card_metadata_columns::id
                    .eq(card_collection_bookmarks_columns::card_metadata_id)
                    .and(card_collection_bookmarks_columns::collection_id.eq(collection_id)),
            ),
        )
        .filter(
            card_metadata_columns::private
                .eq(false)
                .or(card_metadata_columns::author_id
                    .eq(current_user_id.unwrap_or(uuid::Uuid::nil())))
                .and(
                    second_join
                        .field(schema::card_metadata::qdrant_point_id)
                        .is_not_null()
                        .or(card_metadata_columns::qdrant_point_id.is_not_null()),
                ),
        )
        .filter(card_collection_bookmarks_columns::collection_id.eq(collection_id))
        .select((
            (
                card_metadata_columns::id,
                card_metadata_columns::content,
                card_metadata_columns::link,
                card_metadata_columns::author_id,
                card_metadata_columns::qdrant_point_id,
                card_metadata_columns::created_at,
                card_metadata_columns::updated_at,
                card_metadata_columns::tag_set,
                card_metadata_columns::card_html,
                card_metadata_columns::private,
                card_metadata_columns::metadata,
                card_metadata_columns::tracking_id,
                card_metadata_columns::time_stamp,
                sql::<Nullable<Double>>(
                    "paradedb.rank_bm25(card_metadata.ctid)::double precision as rank",
                ),
                sql::<Int8>("count(*) OVER() AS full_count"),
            ),
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

    query = query.filter(sql::<Bool>(
        format!("card_metadata @@@ '{}'", user_query).as_str(),
    ));

    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();

    if let Some(tag) = tag_set_inner.get(0) {
        query = query.filter(card_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }
    for tag in tag_set_inner.iter().skip(1) {
        query = query.or_filter(card_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }

    if let Some(link_inner) = link_inner.get(0) {
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
                        if let Some(first_val) = arr.get(0) {
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
        sql::<Text>("rank DESC"),
    ));

    query = query
        .limit(10)
        .offset(((page - 1) * 10).try_into().unwrap_or(0));

    let searched_cards: Vec<(FullTextSearchResult, Option<uuid::Uuid>)> =
        query.load(&mut conn).map_err(|_| DefaultError {
            message: "Failed to load trigram searched cards",
        })?;

    let card_metadata_with_upvotes_and_files = get_metadata_query(
        searched_cards
            .iter()
            .map(|card| card.0.clone())
            .collect::<Vec<FullTextSearchResult>>(),
        current_user_id,
        conn,
    )
    .map_err(|_| DefaultError {
        message: "Failed to load searched cards",
    })?;

    let total_count = if searched_cards.is_empty() {
        0
    } else {
        (searched_cards
            .get(0)
            .expect("searched_cards len should be at least 1")
            .0
            .count as f64
            / 10.0)
            .ceil() as i64
    };

    Ok(FullTextSearchCardQueryResult {
        search_results: card_metadata_with_upvotes_and_files,
        total_card_pages: total_count,
    })
}

pub async fn search_semantic_cards(
    data: web::Json<SearchCardData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    current_user_id: Option<uuid::Uuid>,
    app_mutex: web::Data<AppMutexStore>,
) -> Result<SearchCardQueryResponseBody, actix_web::Error> {
    let embedding_vector = create_embedding(&data.content, app_mutex).await?;

    let search_card_query_results = retrieve_qdrant_points_query(
        embedding_vector,
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        current_user_id,
        parsed_query,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let (metadata_cards, collided_cards) = web::block(move || {
        get_metadata_and_collided_cards_from_point_ids_query(point_ids, current_user_id, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut card: CardMetadataWithVotesWithScore = match metadata_cards
                .iter()
                .find(|metadata_card| metadata_card.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_card) => metadata_card.clone(),
                None => CardMetadataWithVotesWithScore {
                    id: uuid::Uuid::default(),
                    author: None,
                    qdrant_point_id: uuid::Uuid::default(),
                    total_upvotes: 0,
                    total_downvotes: 0,
                    vote_by_current_user: None,
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    private: false,
                    score: Some(0.0),
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
            let mut collided_cards: Vec<CardMetadataWithVotesWithScore> = collided_cards
                .iter()
                .filter(|card| card.qdrant_id == search_result.point_id)
                .map(|card| card.metadata.clone())
                .collect();

            if !card.private
                || card
                    .clone()
                    .author
                    .is_some_and(|author| Some(author.id) == current_user_id)
            {
                collided_cards.insert(0, card);
            }

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

pub async fn search_full_text_cards(
    data: web::Json<SearchCardData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    current_user_id: Option<uuid::Uuid>,
) -> Result<SearchCardQueryResponseBody, actix_web::Error> {
    let pool1 = pool.clone();
    let user_query = data.content.split_whitespace().join(" AND ");
    let data_inner = data.clone();
    let search_card_query_results = web::block(move || {
        search_full_text_card_query(
            user_query,
            page,
            pool,
            current_user_id,
            data_inner.filters,
            data_inner.link,
            data_inner.tag_set,
            data_inner.time_range,
            parsed_query,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))
    .await?;

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.qdrant_point_id)
        .collect::<Vec<uuid::Uuid>>();

    let collided_cards =
        web::block(move || get_collided_cards_query(point_ids, current_user_id, pool1))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut full_text_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result: &CardMetadataWithVotesWithScore| {
            let mut collided_cards: Vec<CardMetadataWithVotesWithScore> = collided_cards
                .iter()
                .filter(|card| {
                    card.1 == search_result.qdrant_point_id && card.0.id != search_result.id
                })
                .map(|card| card.0.clone())
                .collect();

            // de-duplicate collided cards by removing cards with the same metadata: Option<serde_json::Value>
            let mut seen_metadata = HashSet::new();
            let mut i = 0;
            while i < collided_cards.len() {
                let metadata_string = serde_json::to_string(&collided_cards[i].metadata).unwrap();

                if seen_metadata.contains(&metadata_string) {
                    collided_cards.remove(i);
                } else {
                    seen_metadata.insert(metadata_string);
                    i += 1;
                }
            }
            let highlighted_sentence =
                &find_relevant_sentence(search_result.clone(), data.content.clone())
                    .unwrap_or(search_result.clone());
            collided_cards.insert(0, highlighted_sentence.clone());

            ScoreCardDTO {
                metadata: collided_cards,
                score: search_result.score.unwrap_or(0.0),
            }
        })
        .collect();

    // order full_text_cards by score desc
    full_text_cards.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(SearchCardQueryResponseBody {
        score_cards: full_text_cards,
        total_card_pages: search_card_query_results.total_card_pages,
    })
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
    let scores = Python::with_gil(|py| {
        let token_kwargs = PyDict::new(py);
        token_kwargs.set_item("return_tensors", "pt").map_err(|e| {
            ServiceError::BadRequest(format!("Could not set return_tensors: {}", e))
        })?;
        token_kwargs
            .set_item("truncation", true)
            .map_err(|e| ServiceError::BadRequest(format!("Could not set trucation: {}", e)))?;
        token_kwargs
            .set_item("padding", "max_length")
            .map_err(|e| ServiceError::BadRequest(format!("Could not set padding: {}", e)))?;
        token_kwargs
            .set_item("max_length", 512)
            .map_err(|e| ServiceError::BadRequest(format!("Could not set max_length: {}", e)))?;

        let tokenized_inputs = cross_encoder_init
            .tokenizer
            .call_method::<&str, (pyo3::Py<pyo3::PyAny>,)>(
                py,
                "batch_encode_plus",
                (paired_results.into_py(py),),
                Some(token_kwargs),
            )
            .map_err(|e| ServiceError::BadRequest(format!("Could not tokenize inputs: {}", e)))?
            .into_ref(py);

        let model_kwargs = PyDict::new(py);
        model_kwargs
            .set_item("return_dict", true)
            .map_err(|e| ServiceError::BadRequest(format!("Could not set return_dict: {}", e)))?;

        let output = cross_encoder_init
            .model
            .call_method::<&str, (
                pyo3::Py<pyo3::PyAny>,
                pyo3::Py<pyo3::PyAny>,
                pyo3::Py<pyo3::PyAny>,
            )>(
                py,
                "forward",
                (
                    tokenized_inputs.get_item("input_ids").unwrap().into(),
                    tokenized_inputs.get_item("token_type_ids").unwrap().into(),
                    tokenized_inputs.get_item("attention_mask").unwrap().into(),
                ),
                Some(model_kwargs),
            )
            .map_err(|e| ServiceError::BadRequest(format!("Could not run model: {}", e)))?
            .into_ref(py);

        let scores = output
            .getattr("logits")
            .map_err(|e| ServiceError::BadRequest(format!("Could not get logits: {}", e)))?
            .call_method0("tolist")
            .map_err(|e| ServiceError::BadRequest(format!("Could not get tolist: {}", e)))?;

        Ok::<Vec<f32>, ServiceError>(
            scores
                .extract::<Vec<Vec<f32>>>()
                .unwrap()
                .into_iter()
                .flatten()
                .collect(),
        )
    })?;

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

pub async fn search_hybrid_cards(
    data: web::Json<SearchCardData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    current_user_id: Option<uuid::Uuid>,
    cross_encoder_init: web::Data<CrossEncoder>,
    app_mutex: web::Data<AppMutexStore>,
) -> Result<SearchCardQueryResponseBody, actix_web::Error> {
    let embedding_vector = create_embedding(&data.content, app_mutex).await?;
    let pool1 = pool.clone();

    let search_card_query_results = retrieve_qdrant_points_query(
        embedding_vector,
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        current_user_id,
        parsed_query.clone(),
    );

    let full_text_handler_results = search_full_text_cards(
        web::Json(data.clone()),
        parsed_query,
        page,
        pool,
        current_user_id,
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

    let (metadata_cards, collided_cards) = web::block(move || {
        get_metadata_and_collided_cards_from_point_ids_query(point_ids, current_user_id, pool1)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let semantic_score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut card: CardMetadataWithVotesWithScore = match metadata_cards
                .iter()
                .find(|metadata_card| metadata_card.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_card) => metadata_card.clone(),
                None => CardMetadataWithVotesWithScore {
                    id: uuid::Uuid::default(),
                    author: None,
                    qdrant_point_id: uuid::Uuid::default(),
                    total_upvotes: 0,
                    total_downvotes: 0,
                    vote_by_current_user: None,
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    private: false,
                    score: Some(0.0),
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
            let mut collided_cards: Vec<CardMetadataWithVotesWithScore> = collided_cards
                .iter()
                .filter(|card| card.qdrant_id == search_result.point_id)
                .map(|card| card.metadata.clone())
                .collect();

            if !card.private
                || card
                    .clone()
                    .author
                    .is_some_and(|author| Some(author.id) == current_user_id)
            {
                collided_cards.insert(0, card);
            }

            ScoreCardDTO {
                metadata: collided_cards,
                score: search_result.score as f64 * 0.5,
            }
        })
        .collect();

    if data.cross_encoder.unwrap_or(false) {
        let combined_results = semantic_score_cards
            .into_iter()
            .chain(full_text_handler_results.score_cards.clone().into_iter())
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

pub async fn search_semantic_collections(
    data: web::Json<SearchCollectionsData>,
    parsed_query: ParsedQuery,
    collection: CardCollection,
    page: u64,
    pool: web::Data<Pool>,
    current_user_id: Option<uuid::Uuid>,
    app_mutex: web::Data<AppMutexStore>,
) -> Result<SearchCollectionsResult, actix_web::Error> {
    let embedding_vector: Vec<f32> = create_embedding(&data.content, app_mutex).await?;
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
        current_user_id,
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

    let metadata_cards =
        web::block(move || get_metadata_from_point_ids(point_ids, current_user_id, pool3))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let collided_cards =
        web::block(move || get_collided_cards_query(point_ids_1, current_user_id, pool1))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut card: CardMetadataWithVotesWithScore = match metadata_cards
                .iter()
                .find(|metadata_card| metadata_card.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_card) => metadata_card.clone(),
                None => CardMetadataWithVotesWithScore {
                    id: uuid::Uuid::default(),
                    author: None,
                    qdrant_point_id: uuid::Uuid::default(),
                    total_upvotes: 0,
                    total_downvotes: 0,
                    vote_by_current_user: None,
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    private: false,
                    score: Some(0.0),
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

            let mut collided_cards: Vec<CardMetadataWithVotesWithScore> = collided_cards
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

pub async fn search_full_text_collections(
    data: web::Json<SearchCollectionsData>,
    parsed_query: ParsedQuery,
    collection: CardCollection,
    page: u64,
    pool: web::Data<Pool>,
    current_user_id: Option<uuid::Uuid>,
) -> Result<SearchCollectionsResult, actix_web::Error> {
    let data_inner = data.clone();
    let pool1 = pool.clone();
    let pool2 = pool.clone();

    let search_card_query_results = web::block(move || {
        search_full_text_collection_query(
            data_inner.content.clone(),
            page,
            pool2,
            current_user_id,
            data_inner.filters.clone(),
            data_inner.link.clone(),
            data_inner.tag_set.clone(),
            data_inner.collection_id,
            parsed_query,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.qdrant_point_id)
        .collect::<Vec<uuid::Uuid>>();

    let collided_cards =
        web::block(move || get_collided_cards_query(point_ids, current_user_id, pool1))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let full_text_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut collided_cards: Vec<CardMetadataWithVotesWithScore> = collided_cards
                .iter()
                .filter(|card| {
                    card.1 == search_result.qdrant_point_id && card.0.id != search_result.id
                })
                .map(|card| card.0.clone())
                .collect();

            // de-duplicate collided cards by removing cards with the same metadata: Option<serde_json::Value>
            let mut seen_metadata = HashSet::new();
            let mut i = 0;
            while i < collided_cards.len() {
                let metadata_string = serde_json::to_string(&collided_cards[i].metadata).unwrap();

                if seen_metadata.contains(&metadata_string) {
                    collided_cards.remove(i);
                } else {
                    seen_metadata.insert(metadata_string);
                    i += 1;
                }
            }
            let highlighted_sentence =
                &find_relevant_sentence(search_result.clone(), data.content.clone())
                    .unwrap_or(search_result.clone());

            collided_cards.insert(0, highlighted_sentence.clone());

            ScoreCardDTO {
                metadata: collided_cards,
                score: search_result.score.unwrap_or(0.0),
            }
        })
        .collect();
    Ok(SearchCollectionsResult {
        bookmarks: full_text_cards,
        collection,
        total_pages: search_card_query_results.total_card_pages,
    })
}
