use super::{
    auth_handler::LoggedUser,
    chunk_handler::{ChunkFilter, ScoringOptions},
};
use crate::{
    data::models::Templates, operators::dataset_operator::get_dataset_by_tracking_id_unsafe_query,
};
use crate::{
    data::models::{DatasetConfiguration, Pool, SearchMethod, SortOptions, TypoOptions},
    errors::ServiceError,
    get_env,
    operators::{
        dataset_operator::get_dataset_by_id_query, organization_operator::get_org_from_id_query,
    },
};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use minijinja::context;
use serde::{Deserialize, Serialize};
use std::env;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
pub enum PublicPageTheme {
    #[default]
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

// Duplicate of SearchChunksReqPayload but without "query"
#[derive(Serialize, Clone, Debug, ToSchema, Deserialize)]
#[schema(example = json!({
    "search_type": "semantic",
    "filters": {
        "should": [
            {
                "field": "metadata.key1",
                "match": ["value1", "value2"],
            }
        ],
        "must": [
            {
                "field": "num_value",
                "range": {
                    "gte": 0.0,
                    "lte": 1.0,
                    "gt": 0.0,
                    "lt": 1.0
                }
            }
        ],
        "must_not": [
            {
                "field": "metadata.key3",
                "match": ["value5", "value6"],
            }
        ]
    },
    "score_threshold": 0.5
}))]
pub struct PublicPageSearchOptions {
    /// Can be either "semantic", "fulltext", "hybrid, or "bm25". If specified as "hybrid", it will pull in one page of both semantic and full-text results then re-rank them using scores from a cross encoder model. "semantic" will pull in one page of the nearest cosine distant vectors. "fulltext" will pull in one page of full-text results based on SPLADE. "bm25" will get one page of results scored using BM25 with the terms OR'd together.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_type: Option<SearchMethod>,
    /// Page of chunks to fetch. Page is 1-indexed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    /// Page size is the number of chunks to fetch. This can be used to fetch more than 10 chunks at a time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u64>,
    /// Get total page count for the query accounting for the applied filters. Defaults to false, but can be set to true when the latency penalty is acceptable (typically 50-200ms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_total_pages: Option<bool>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<ChunkFilter>,
    /// Sort Options lets you specify different methods to rerank the chunks in the result set. If not specified, this defaults to the score of the chunks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_options: Option<SortOptions>,
    /// Scoring options provides ways to modify the sparse or dense vector created for the query in order to change how potential matches are scored. If not specified, this defaults to no modifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoring_options: Option<ScoringOptions>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold for cosine distance metric. For Manhattan Distance, Euclidean Distance, and Dot Product, it will filter out scores above the threshold distance. This threshold applies before weight and bias modifications. If not specified, this defaults to no threshold. A threshold of 0 will default to no threshold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f32>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typically 10-50ms). Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slim_chunks: Option<bool>,
    /// Set content_only to true to only returning the chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typically 10-50ms). Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_only: Option<bool>,
    /// If true, quoted and - prefixed words will be parsed from the queries and used as required and negated words respectively. Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_quote_negated_terms: Option<bool>,
    /// If true, stop words (specified in server/src/stop-words.txt in the git repo) will be removed. Queries that are entirely stop words will be preserved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_stop_words: Option<bool>,
    /// User ID is the id of the user who is making the request. This is used to track user interactions with the search results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Typo options lets you specify different methods to handle typos in the search query. If not specified, this defaults to no typo handling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typo_options: Option<TypoOptions>,
    /// Enables autocomplete on the search modal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_autocomplete: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct HeroPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hero_pattern_svg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hero_pattern_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground_opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct PublicPageTabMessage {
    title: String,
    tab_inner_html: String,
    show_component_code: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenGraphMetadata {
    title: Option<String>,
    image: Option<String>,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ButtonTrigger {
    selector: String,
    mode: String,
    remove_triggers: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct SingleProductOptions {
    product_tracking_id: Option<String>,
    group_tracking_id: Option<String>,
    product_name: Option<String>,
    product_description_html: Option<String>,
    product_primary_image_url: Option<String>,
    rec_search_query: Option<String>,
    product_questions: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct PublicPageTag {
    tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_class_name: Option<String>,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RelevanceToolCallOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_message_text_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_images: Option<bool>,
    pub tool_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low_description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct PriceToolCallOptions {
    pub tool_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_price_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price_description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct TagProp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<RangeSliderConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RangeSliderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct FilterSidebarSection {
    pub key: String,
    pub filter_key: String,
    pub title: String,
    pub selection_type: String,
    pub filter_type: String,
    pub options: Vec<TagProp>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct SidebarFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<FilterSidebarSection>>,
}
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchPageProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_sidebar_props: Option<SidebarFilters>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct PublicPageParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<PublicPageTag>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relevance_tool_call_options: Option<RelevanceToolCallOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_tool_call_options: Option<PriceToolCallOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_queries: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followup_questions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<PublicPageTheme>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_options: Option<PublicPageSearchOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub for_brand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_logo_img_src_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nav_logo_img_src_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_search_queries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_ai_questions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_search_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_group_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_switching_modes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_floating_button: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floating_button_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floating_button_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floating_search_icon_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_floating_search_icon: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_floating_input: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_triggers: Option<Vec<ButtonTrigger>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debounce_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hero_pattern: Option<HeroPattern>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_messages: Option<Vec<PublicPageTabMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_graph_metadata: Option<OpenGraphMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_product_options: Option<SingleProductOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_links_in_new_tab: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_linked_in_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_drawn_text: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_pagefind: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_test_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_suggestions: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_image_question: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_local: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_result_highlights: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_page_props: Option<SearchPageProps>,
}

#[utoipa::path(
    get,
    path = "/public_page/{dataset_id}",
    context_path = "/api",
    tag = "Public",
    responses(
        (status = 200, description = "Public Page associated to the dataset"),
        (status = 400, description = "Service error relating to loading the public page", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("dataset_id" = String, Path, description = "The id or tracking_id of the dataset you want to get the demo page for."),
    ),
)]
pub async fn public_page(
    dataset_id: web::Path<String>,
    pool: web::Data<Pool>,
    templates: Templates<'_>,
    req: HttpRequest,
) -> Result<HttpResponse, ServiceError> {
    let dataset_id = dataset_id.into_inner();
    let dataset: crate::data::models::Dataset = match uuid::Uuid::parse_str(&dataset_id) {
        Ok(uuid) => get_dataset_by_id_query(uuid, pool.clone()).await?,
        Err(_) => get_dataset_by_tracking_id_unsafe_query(dataset_id, pool.clone()).await?,
    };
    let org_sub_plan = get_org_from_id_query(dataset.organization_id, pool).await?;

    let config = DatasetConfiguration::from_json(dataset.server_configuration);

    let base_server_url = get_env!(
        "BASE_SERVER_URL",
        "Server hostname for OpenID provider must be set"
    );

    let logged_in = req.extensions().get::<LoggedUser>().is_some();
    let dashboard_url =
        env::var("ADMIN_DASHBOARD_URL").unwrap_or("https://dashboard.trieve.ai".to_string());

    let search_component_url = if config
        .clone()
        .PUBLIC_DATASET
        .extra_params
        .unwrap_or_default()
        .is_test_mode
        .unwrap_or(false)
    {
        "https://cdn.trieve.ai/beta/search-component".to_string()
    } else if config
        .clone()
        .PUBLIC_DATASET
        .extra_params
        .unwrap_or_default()
        .use_local
        .unwrap_or(false)
    {
        "http://localhost:8000/dist".to_string()
    } else {
        std::env::var("SEARCH_COMPONENT_URL")
            .unwrap_or("https://search-component.trieve.ai/dist".to_string())
    };

    if config.PUBLIC_DATASET.enabled {
        let templ = templates.get_template("page.html").unwrap();

        let hero_pattern = config
            .PUBLIC_DATASET
            .extra_params
            .as_ref()
            .and_then(|params| params.hero_pattern.clone());

        let body_style = hero_pattern
            .as_ref()
            .and_then(|p| p.hero_pattern_svg.as_ref())
            .map(|url| format!("background-image: url('{url}')"));
        let background_color = hero_pattern
            .as_ref()
            .and_then(|p| p.background_color.as_ref())
            .map(|color| format!("background-color: {color}"));

        let tabs = config
            .PUBLIC_DATASET
            .extra_params
            .as_ref()
            .cloned()
            .unwrap_or_default()
            .tab_messages
            .clone()
            .unwrap_or_default();

        let partner_configuration = org_sub_plan.organization.partner_configuration;

        let response_body = templ
            .render(context! {
                logged_in,
                dashboard_url,
                background_color,
                search_component_url,
                has_hero_pattern => hero_pattern.is_some(),
                body_style,
                tabs,
                partner_configuration,
                params => PublicPageParameters {
                    dataset_id: Some(dataset.id),
                    base_url: Some(base_server_url.to_string()),
                    api_key: Some(config.PUBLIC_DATASET.api_key.unwrap_or_default()),
                    ..config.PUBLIC_DATASET.extra_params.unwrap_or_default()
                }
            })
            .map_err(|e| {
                log::error!("Error rendering template: {:?}", e);
                ServiceError::InternalServerError("Error rendering template".to_string())
            })?;

        Ok(HttpResponse::Ok().body(response_body))
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}
