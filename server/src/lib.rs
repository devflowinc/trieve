#![allow(clippy::get_first)]
#![allow(deprecated)]

#[macro_use]
extern crate diesel;

use crate::{
    errors::{custom_json_error_handler, ServiceError},
    handlers::auth_handler::build_oidc_client,
    handlers::metrics_handler::Metrics,
    operators::{
        qdrant_operator::create_new_qdrant_collection_query, user_operator::create_default_user,
    },
};
use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::RedisSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{Key, SameSite},
    middleware::{Compress, Logger},
    web::{self, PayloadConfig},
    App, HttpServer,
};
use chm::tools::migrations::{run_pending_migrations, SetupArgs};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::ManagerConfig;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use openssl::ssl::SslVerifyMode;
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use tracing_subscriber::{prelude::*, EnvFilter, Layer};
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

pub mod data;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod operators;
pub mod randutil;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
pub const SECONDS_IN_MINUTE: u64 = 60;
pub const SECONDS_IN_HOUR: u64 = 60 * SECONDS_IN_MINUTE;
pub const SECONDS_IN_DAY: u64 = 24 * SECONDS_IN_HOUR;

fn run_migrations(url: &str) {
    use diesel::prelude::*;

    // Run migrations in sync just because the async_diesel_migrations crate isn't very popular
    // This is an option but I exceeded my timebox
    // https://github.com/weiznich/diesel_async/blob/main/examples/postgres/run-pending-migrations-with-rustls/src/main.rs

    let mut conn = diesel::pg::PgConnection::establish(url).expect("Failed to connect to database");
    // &mut impl MigrationHarness<diesel::pg::Pg>
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}

pub fn establish_connection(
    config: &str,
) -> BoxFuture<diesel::ConnectionResult<diesel_async::AsyncPgConnection>> {
    let fut = async {
        let mut tls = SslConnector::builder(SslMethod::tls()).unwrap();

        tls.set_verify(SslVerifyMode::NONE);
        let tls_connector = MakeTlsConnector::new(tls.build());

        let (client, conn) = tokio_postgres::connect(config, tls_connector)
            .await
            .map_err(|e| diesel::ConnectionError::BadConnection(e.to_string()))?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("Database connection: {e}");
            }
        });
        diesel_async::AsyncPgConnection::try_from(client).await
    };
    fut.boxed()
}

use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

#[macro_export]
#[cfg(not(feature = "runtime-env"))]
macro_rules! get_env {
    ($name:expr, $message:expr) => {
        env!($name, $message)
    };
}

#[macro_export]
#[cfg(feature = "runtime-env")]
macro_rules! get_env {
    ($name:expr, $message:expr) => {{
        lazy_static::lazy_static! {
            static ref ENV_VAR: String = {
                std::env::var($name).expect($message)
            };
        }
        ENV_VAR.as_str()
    }};
}
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi
            .components
            .as_mut()
            .expect("Safe to expect since the component was already registered");
        components.add_security_scheme(
            "ApiKey",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
        );
        components.add_security_scheme(
            "X-API-KEY",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-KEY"))),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Trieve API",
        description = "Trieve OpenAPI Specification. This document describes all of the operations available through the Trieve API.", 
        contact(
            name = "Trieve Team",
            url = "https://trieve.ai",
            email = "developers@trieve.ai",
        ),
        license(
            name = "BSL",
            url = "https://github.com/devflowinc/trieve/blob/main/LICENSE.txt",
        ),
        version = "0.11.7",
    ),
    servers(
        (url = "https://api.trieve.ai",
        description = "Production server"),
        (url = "http://localhost:8090",
        description = "Local development server"),
    ),
    modifiers(&SecurityAddon),
    paths(
        handlers::invitation_handler::post_invitation,
        handlers::auth_handler::login,
        handlers::auth_handler::logout,
        handlers::auth_handler::get_me,
        handlers::auth_handler::callback,
        handlers::auth_handler::health_check,
        handlers::topic_handler::create_topic,
        handlers::topic_handler::delete_topic,
        handlers::topic_handler::update_topic,
        handlers::topic_handler::get_all_topics_for_owner_id,
        handlers::message_handler::create_message,
        handlers::message_handler::get_all_topic_messages,
        handlers::message_handler::edit_message,
        handlers::message_handler::regenerate_message,
        handlers::message_handler::regenerate_message_patch,
        handlers::message_handler::get_suggested_queries,
        handlers::chunk_handler::create_chunk,
        handlers::chunk_handler::update_chunk,
        handlers::chunk_handler::delete_chunk,
        handlers::chunk_handler::get_recommended_chunks,
        handlers::chunk_handler::update_chunk_by_tracking_id,
        handlers::chunk_handler::search_chunks,
        handlers::chunk_handler::count_chunks,
        handlers::chunk_handler::generate_off_chunks,
        handlers::chunk_handler::get_chunk_by_tracking_id,
        handlers::chunk_handler::get_chunks_by_tracking_ids,
        handlers::chunk_handler::delete_chunk_by_tracking_id,
        handlers::chunk_handler::get_chunk_by_id,
        handlers::chunk_handler::autocomplete,
        handlers::chunk_handler::get_chunks_by_ids,
        handlers::chunk_handler::scroll_dataset_chunks,
        handlers::user_handler::update_user,
        handlers::user_handler::set_user_api_key,
        handlers::user_handler::delete_user_api_key,
        handlers::group_handler::search_over_groups,
        handlers::group_handler::get_recommended_groups,
        handlers::group_handler::get_groups_for_dataset,
        handlers::group_handler::create_chunk_group,
        handlers::group_handler::delete_chunk_group,
        handlers::group_handler::update_chunk_group,
        handlers::group_handler::add_chunk_to_group,
        handlers::group_handler::get_chunk_group,
        handlers::group_handler::remove_chunk_from_group,
        handlers::group_handler::get_chunks_in_group,
        handlers::group_handler::get_groups_for_chunks,
        handlers::group_handler::get_group_by_tracking_id,
        handlers::group_handler::delete_group_by_tracking_id,
        handlers::group_handler::update_group_by_tracking_id,
        handlers::group_handler::add_chunk_to_group_by_tracking_id,
        handlers::group_handler::get_chunks_in_group_by_tracking_id,
        handlers::group_handler::search_within_group,
        handlers::file_handler::get_dataset_files_handler,
        handlers::file_handler::upload_file_handler,
        handlers::file_handler::get_file_handler,
        handlers::file_handler::delete_file_handler,
        handlers::event_handler::get_events,
        handlers::organization_handler::create_organization,
        handlers::organization_handler::get_organization,
        handlers::organization_handler::update_organization,
        handlers::organization_handler::delete_organization,
        handlers::organization_handler::get_organization_usage,
        handlers::organization_handler::get_organization_users,
        handlers::organization_handler::update_all_org_dataset_configs,
        handlers::dataset_handler::create_dataset,
        handlers::dataset_handler::update_dataset,
        handlers::dataset_handler::delete_dataset,
        handlers::dataset_handler::delete_dataset_by_tracking_id,
        handlers::dataset_handler::get_dataset,
        handlers::dataset_handler::get_usage_by_dataset_id,
        handlers::dataset_handler::get_datasets_from_organization,
        handlers::dataset_handler::clear_dataset,
        handlers::stripe_handler::direct_to_payment_link,
        handlers::stripe_handler::cancel_subscription,
        handlers::stripe_handler::update_subscription_plan,
        handlers::stripe_handler::get_all_plans,
        handlers::stripe_handler::get_all_invoices,
        handlers::stripe_handler::create_setup_checkout_session,
        handlers::analytics_handler::get_cluster_analytics,
        handlers::analytics_handler::get_rag_analytics,
        handlers::analytics_handler::get_search_analytics,
        handlers::analytics_handler::get_recommendation_analytics,
        handlers::analytics_handler::send_ctr_data,
        handlers::analytics_handler::get_ctr_analytics,
        handlers::metrics_handler::get_metrics,
    ),
    components(
        schemas(
            handlers::auth_handler::AuthQuery,
            handlers::topic_handler::CreateTopicReqPayload,
            handlers::topic_handler::DeleteTopicData,
            handlers::topic_handler::UpdateTopicReqPayload,
            handlers::message_handler::CreateMessageReqPayload,
            handlers::message_handler::RegenerateMessageReqPayload,
            handlers::message_handler::EditMessageReqPayload,
            handlers::message_handler::SuggestedQueriesReqPayload,
            handlers::message_handler::SuggestedQueriesResponse,
            handlers::chunk_handler::FullTextBoost,
            handlers::chunk_handler::ChunkReqPayload,
            handlers::chunk_handler::CreateChunkReqPayloadEnum,
            handlers::chunk_handler::CreateSingleChunkReqPayload,
            handlers::chunk_handler::SearchResponseBody,
            handlers::chunk_handler::SearchResponseTypes,
            handlers::chunk_handler::CreateBatchChunkReqPayload,
            handlers::chunk_handler::SingleQueuedChunkResponse,
            handlers::chunk_handler::BatchQueuedChunkResponse,
            handlers::chunk_handler::ReturnQueuedChunk,
            handlers::chunk_handler::RecommendChunksResponseBody,
            handlers::chunk_handler::RecommendResponseTypes,
            handlers::chunk_handler::UpdateChunkReqPayload,
            handlers::chunk_handler::RecommendChunksRequest,
            handlers::chunk_handler::UpdateChunkByTrackingIdData,
            handlers::chunk_handler::SearchChunkQueryResponseBody,
            handlers::chunk_handler::GenerateChunksRequest,
            handlers::chunk_handler::SearchChunksReqPayload,
            handlers::chunk_handler::CountChunksReqPayload,
            handlers::chunk_handler::CountChunkQueryResponseBody,
            handlers::chunk_handler::AutocompleteReqPayload,
            handlers::chunk_handler::SearchChunkQueryResponseBody,
            handlers::chunk_handler::ChunkFilter,
            handlers::chunk_handler::ChunkReturnTypes,
            handlers::chunk_handler::GetChunksData,
            handlers::chunk_handler::GetTrackingChunksData,
            handlers::chunk_handler::SemanticBoost,
            handlers::chunk_handler::ChunkReturnTypes,
            handlers::chunk_handler::ScrollChunksReqPayload,
            handlers::chunk_handler::ScrollChunksResponseBody,
            handlers::group_handler::RecommendGroupsReqPayload,
            handlers::group_handler::RecommendGroupsResponse,
            handlers::group_handler::SearchWithinGroupReqPayload,
            handlers::group_handler::SearchOverGroupsReqPayload,
            handlers::group_handler::SearchWithinGroupResponseBody,
            handlers::group_handler::SearchGroupResponseTypes,
            handlers::group_handler::SearchWithinGroupResults,
            handlers::group_handler::GroupData,
            handlers::group_handler::CreateChunkGroupReqPayloadEnum,
            handlers::group_handler::CreateBatchChunkGroupReqPayload,
            handlers::group_handler::CreateSingleChunkGroupReqPayload,
            handlers::group_handler::ChunkGroups,
            handlers::group_handler::CreateChunkGroupResponseEnum,
            handlers::group_handler::UpdateChunkGroupReqPayload,
            handlers::group_handler::AddChunkToGroupReqPayload,
            handlers::group_handler::GetGroupsForChunksReqPayload,
            handlers::group_handler::GroupsBookmarkQueryResult,
            handlers::group_handler::GetChunksInGroupsResponseBody,
            handlers::group_handler::GetChunksInGroupResponse,
            handlers::group_handler::RemoveChunkFromGroupReqPayload,
            handlers::group_handler::UpdateGroupByTrackingIDReqPayload,
            handlers::group_handler::AddChunkToGroupReqPayload,
            handlers::group_handler::RecommendGroupsResponseBody,
            handlers::user_handler::UpdateUserOrgRoleData,
            handlers::user_handler::SetUserApiKeyRequest,
            handlers::user_handler::SetUserApiKeyResponse,
            handlers::user_handler::DeleteUserApiKeyRequest,
            operators::group_operator::GroupsForChunk,
            handlers::file_handler::UploadFileReqPayload,
            handlers::file_handler::UploadFileResult,
            handlers::invitation_handler::InvitationData,
            handlers::event_handler::GetEventsData,
            handlers::organization_handler::CreateOrganizationReqPayload,
            handlers::organization_handler::UpdateOrganizationReqPayload,
            handlers::organization_handler::UpdateAllOrgDatasetConfigsReqPayload,
            operators::event_operator::EventReturn,
            operators::search_operator::DeprecatedSearchOverGroupsResponseBody,
            operators::search_operator::GroupScoreChunk,
            operators::search_operator::SearchOverGroupsResults,
            operators::search_operator::SearchOverGroupsResponseBody,
            operators::search_operator::SearchOverGroupsResponseTypes,
            handlers::dataset_handler::CreateDatasetRequest,
            handlers::dataset_handler::UpdateDatasetRequest,
            handlers::dataset_handler::GetDatasetsPagination,
            data::models::DatasetConfigurationDTO,
            operators::analytics_operator::HeadQueryResponse,
            operators::analytics_operator::LatencyGraphResponse,
            operators::analytics_operator::RPSGraphResponse,
            operators::analytics_operator::RagQueryResponse,
            operators::analytics_operator::SearchClusterResponse,
            operators::analytics_operator::SearchQueryResponse,
            operators::analytics_operator::RecommendationsEventResponse,
            operators::analytics_operator::QueryCountResponse,
            operators::analytics_operator::CTRSearchQueryWithClicksResponse,
            operators::analytics_operator::CTRSearchQueryWithoutClicksResponse,
            operators::analytics_operator::CTRRecommendationsWithClicksResponse,
            operators::analytics_operator::CTRRecommendationsWithoutClicksResponse,
            handlers::analytics_handler::CTRDataRequestBody,
            operators::chunk_operator::HighlightStrategy,
            handlers::stripe_handler::CreateSetupCheckoutSessionResPayload,
            data::models::DateRange,
            data::models::FieldCondition,
            data::models::Range,
            data::models::MatchCondition,
            data::models::SearchQueryEvent,
            data::models::SearchTypeCount,
            data::models::SearchClusterTopics,
            data::models::SearchLatencyGraph,
            data::models::SearchAnalyticsFilter,
            data::models::RAGAnalyticsFilter,
            data::models::RagTypes,
            data::models::RagQueryEvent,
            data::models::CountSearchMethod,
            data::models::SearchMethod,
            data::models::SearchType,
            data::models::ApiKeyRespBody,
            data::models::SearchRPSGraph,
            data::models::SearchResultType,
            data::models::RoleProxy,
            data::models::RAGUsageResponse,
            data::models::ClusterAnalytics,
            data::models::RAGAnalytics,
            data::models::SearchAnalytics,
            data::models::ClusterAnalyticsResponse,
            data::models::ClusterAnalyticsFilter,
            data::models::RAGAnalyticsResponse,
            data::models::EventTypeRequest,
            data::models::RecommendationAnalyticsResponse,
            data::models::RecommendationEvent,
            data::models::RecommendationAnalyticsFilter,
            data::models::RecommendationAnalytics,
            data::models::RecommendationType,
            data::models::RecommendType,
            data::models::SlimChunkMetadataWithArrayTagSet,
            data::models::NewChunkMetadataTypes,
            data::models::CTRAnalytics,
            data::models::SearchCTRMetrics,
            data::models::RecommendationCTRMetrics,
            data::models::CTRType,
            data::models::CTRAnalyticsResponse,
            data::models::SearchQueriesWithoutClicksCTRResponse,
            data::models::SearchQueriesWithClicksCTRResponse,
            data::models::RecommendationsWithClicksCTRResponse,
            data::models::RecommendationsWithoutClicksCTRResponse,
            data::models::RecommendationStrategy,
            data::models::ScoreChunk,
            data::models::Granularity,
            data::models::RAGSortBy,
            data::models::SearchSortBy,
            data::models::SortOrder,
            data::models::SearchAnalyticsResponse,
            data::models::DatasetAnalytics,
            data::models::HeadQueries,
            data::models::SlimUser,
            data::models::UserOrganization,
            data::models::QdrantSortBy,
            data::models::SortOptions,
            data::models::LLMOptions,
            data::models::HighlightOptions,
            data::models::SortByField,
            data::models::SortBySearchType,
            data::models::ReRankOptions,
            data::models::Topic,
            data::models::Message,
            data::models::ChunkMetadata,
            data::models::ChatMessageProxy,
            data::models::Event,
            data::models::File,
            data::models::ChunkGroup,
            data::models::ChunkGroupAndFileId,
            data::models::FileDTO,
            data::models::Organization,
            data::models::OrganizationUsageCount,
            data::models::Dataset,
            data::models::DatasetAndUsage,
            data::models::DatasetUsageCount,
            data::models::DatasetDTO,
            data::models::DatasetUsageCount,
            data::models::StripePlan,
            data::models::StripeInvoice,
            data::models::SlimChunkMetadata,
            data::models::RangeCondition,
            data::models::LocationBoundingBox,
            data::models::LocationPolygon,
            data::models::LocationRadius,
            data::models::ChunkMetadataWithScore,
            data::models::SlimChunkMetadataWithScore,
            data::models::GeoInfo,
            data::models::GeoInfoWithBias,
            data::models::GeoTypes,
            data::models::ScoreChunkDTO,
            data::models::ChunkMetadataTypes,
            data::models::ContentChunkMetadata,
            data::models::ChunkMetadataStringTagSet,
            data::models::ConditionType,
            data::models::HasIDCondition,
            errors::ErrorResponseBody,
            middleware::api_version::APIVersion,
        )
    ),
    tags(
        (name = "Invitation", description = "Invitation endpoint. Exists to invite users to an organization."),
        (name = "Auth", description = "Authentication endpoint. Serves to register and authenticate users."),
        (name = "User", description = "User endpoint. Enables you to modify user roles and information."),
        (name = "Organization", description = "Organization endpoint. Enables you to modify organization roles and information."),
        (name = "Dataset", description = "Dataset endpoint. Datasets belong to organizations and hold configuration information for both client and server. Datasets contain chunks and chunk groups."),
        (name = "Chunk", description = "Chunk endpoint. Think of chunks as individual searchable units of information. The majority of your integration will likely be with the Chunk endpoint."),
        (name = "Chunk Group", description = "Chunk groups endpoint. Think of a chunk_group as a bookmark folder within the dataset."),
        (name = "File", description = "File endpoint. When files are uploaded, they are stored in S3 and broken up into chunks with text extraction from Apache Tika. You can upload files of pretty much any type up to 1GB in size. See chunking algorithm details at `docs.trieve.ai` for more information on how chunking works. Improved default chunking is on our roadmap."),
        (name = "Events", description = "Notifications endpoint. Files are uploaded asynchronously and events are sent to the user when the upload is complete."),
        (name = "Topic", description = "Topic chat endpoint. Think of topics as the storage system for gen-ai chat memory. Gen AI messages belong to topics."),
        (name = "Message", description = "Message chat endpoint. Messages are units belonging to a topic in the context of a chat with a LLM. There are system, user, and assistant messages."),
        (name = "Stripe", description = "Stripe endpoint. Used for the managed SaaS version of this app. Eventually this will become a micro-service. Reach out to the team using contact info found at `docs.trieve.ai` for more information."),
        (name = "Health", description = "Health check endpoint. Used to check if the server is up and running."),
        (name = "Metrics", description = "Metrics endpoint. Used to get information for monitoring"),
        (name = "Analytics", description = "Analytics endpoint. Used to get information for search and RAG analytics"),
    ),
)]
pub struct ApiDoc;

#[tracing::instrument]
pub fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let sentry_url = std::env::var("SENTRY_URL");
    let _guard = if let Ok(sentry_url) = sentry_url {
        let guard = sentry::init((
            sentry_url,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                traces_sample_rate: 1.0,
                ..Default::default()
            },
        ));

        tracing_subscriber::Registry::default()
            .with(sentry::integrations::tracing::layer())
            .with(
                tracing_subscriber::fmt::layer().with_filter(
                    EnvFilter::from_default_env()
                        .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
                ),
            )
            .init();

        // std::env::set_var("RUST_BACKTRACE", "1");
        log::info!("Sentry monitoring enabled");
        Some(guard)
    } else {
        tracing_subscriber::Registry::default()
            .with(
                tracing_subscriber::fmt::layer().with_filter(
                    EnvFilter::from_default_env()
                        .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
                ),
            )
            .init();

        None
    };

    let database_url = get_env!("DATABASE_URL", "DATABASE_URL should be set");
    let redis_url = get_env!("REDIS_URL", "REDIS_URL should be set");

    log::info!("Running migrations");
    run_migrations(database_url);

    actix_web::rt::System::new().block_on(async move {
        // create db connection pool
        let mut config = ManagerConfig::default();
        config.custom_setup = Box::new(establish_connection);

        let mgr = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new_with_config(
            database_url,
            config,
        );

        let pool = diesel_async::pooled_connection::deadpool::Pool::builder(mgr)
            .max_size(10)
            .build()
            .unwrap();

        log::info!("Connecting to redis");
        let redis_store = RedisSessionStore::new(redis_url)
            .await
            .expect("Failed to create redis store");

        let redis_manager =
            bb8_redis::RedisConnectionManager::new(redis_url).expect("Failed to connect to redis");

        let redis_connections: u32 = std::env::var("REDIS_CONNECTIONS")
            .unwrap_or("200".to_string())
            .parse()
            .unwrap_or(200);

        let redis_pool = bb8_redis::bb8::Pool::builder()
            .max_size(redis_connections)
            .build(redis_manager)
            .await
            .expect("Failed to create redis pool");

        let oidc_client = build_oidc_client().await;

        let quantize_vectors = std::env::var("QUANTIZE_VECTORS")
            .unwrap_or("false".to_string())
            .parse()
            .unwrap_or(false);

        let replication_factor: u32 = std::env::var("REPLICATION_FACTOR")
            .unwrap_or("2".to_string())
            .parse()
            .unwrap_or(2);

        let vector_sizes: Vec<u64> = std::env::var("VECTOR_SIZES")
            .unwrap_or("384,512,768,1024,1536,3072".to_string())
            .split(',')
            .map(|x| x.parse().ok())
            .collect::<Option<Vec<u64>>>()
            .unwrap_or(vec![384,512,768,1024,1536,3072]);

        let json_cfg = web::JsonConfig::default()
            .limit(134200000)
            .error_handler(custom_json_error_handler);

        if std::env::var("CREATE_QDRANT_COLLECTIONS").unwrap_or("true".to_string()) != "false" {
            log::info!("Creating qdrant collections");
            let _ = create_new_qdrant_collection_query(None, None, quantize_vectors, false, replication_factor, vector_sizes)
                .await
                .map_err(|err| {
                    log::error!("Failed to create new qdrant collection: {:?}", err);
                });
        }

        if std::env::var("ADMIN_API_KEY").is_ok() {
            let _ = create_default_user(
                &std::env::var("ADMIN_API_KEY").expect("ADMIN_API_KEY should be set"),
                web::Data::new(pool.clone()),
            )
            .await
            .map_err(|err| {
                log::error!("Failed to create default user: {:?}", err);
            });
        }


        let clickhouse_client = if std::env::var("USE_ANALYTICS").unwrap_or("false".to_string()).parse().unwrap_or(false) {
            log::info!("Analytics enabled");

            let args  = SetupArgs {
                url: Some(std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string())),
                user: Some(std::env::var("CLICKHOUSE_USER").unwrap_or("default".to_string())),
                password: Some(std::env::var("CLICKHOUSE_PASSWORD").unwrap_or("password".to_string())),
                database: Some(std::env::var("CLICKHOUSE_DB").unwrap_or("default".to_string()))
            };

            let clickhouse_client = clickhouse::Client::default()
                .with_url(args.url.as_ref().unwrap())
                .with_user(args.user.as_ref().unwrap())
                .with_password(args.password.as_ref().unwrap())
                .with_database(args.database.as_ref().unwrap())
                .with_option("async_insert", "1")
                .with_option("wait_for_async_insert", "0");


            let _ = run_pending_migrations(args.clone()).await.map_err(|err| {
                log::error!("Failed to run clickhouse migrations: {:?}", err);
            });
            clickhouse_client
        } else {
            log::info!("Analytics disabled");
            clickhouse::Client::default()
        };


        let metrics = Metrics::new().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create metrics {:?}", e))
        })?;

        HttpServer::new(move || {
            App::new()
                .wrap(Cors::permissive())
                .app_data(PayloadConfig::new(134200000))
                .wrap(middleware::json_middleware::JsonMiddlewareFactory)
                .app_data(json_cfg.clone())
                .app_data(
                    web::PathConfig::default()
                        .error_handler(|err, _req| ServiceError::BadRequest(format!("{}", err)).into()),
                )
                .app_data(web::Data::new(pool.clone()))
                .app_data(web::Data::new(oidc_client.clone()))
                .app_data(web::Data::new(redis_pool.clone()))
                .app_data(web::Data::new(clickhouse_client.clone()))
                .app_data(web::Data::new(metrics.clone()))
                .wrap(sentry_actix::Sentry::new())
                .wrap(middleware::api_version::ApiVersionCheckFactory)
                .wrap(middleware::auth_middleware::AuthMiddlewareFactory)
                .wrap(
                    IdentityMiddleware::builder()
                        .login_deadline(Some(std::time::Duration::from_secs(SECONDS_IN_DAY)))
                        .visit_deadline(Some(std::time::Duration::from_secs(SECONDS_IN_DAY)))
                        .build(),
                )
                .wrap(
                    SessionMiddleware::builder(
                        redis_store.clone(),
                        Key::from(operators::user_operator::SECRET_KEY.as_bytes()),
                    )
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(time::Duration::days(1)),
                    )
                    .cookie_name("vault".to_owned())
                    .cookie_same_site(
                        if std::env::var("COOKIE_SECURE").unwrap_or("false".to_owned()) == "true" {
                            SameSite::None
                        } else {
                            SameSite::Lax
                        },
                    )
                    .cookie_secure(
                        std::env::var("COOKIE_SECURE").unwrap_or("false".to_owned()) == "true",
                    )
                    .cookie_path("/".to_owned())
                    .build(),
                )
                .wrap(
                    // Set up logger, but avoid logging hot status endpoints
                    Logger::default()
                    .exclude("/")
                    .exclude("/api/health")
                    .exclude("/metrics")
                )
                .service(Redoc::with_url("/redoc", ApiDoc::openapi()))
                .service(
                    SwaggerUi::new("/swagger-ui/{_:.*}")
                        .url("/api-docs/openapi.json", ApiDoc::openapi())
                )
                .service(
                    web::redirect("/swagger-ui", "/swagger-ui/")
                )
                .service(
                    web::resource("/auth/cli")
                        .route(web::get().to(handlers::auth_handler::login_cli))
                )
                .service(
                    web::resource("/")
                    .route(web::get().to(handlers::auth_handler::health_check))
                )
                .service(
                    web::resource("/metrics")
                    .route(web::get().to(handlers::metrics_handler::get_metrics))
                )
                // everything under '/api/' route
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/chunks")
                                .service(
                                    web::resource("")
                                        .route(web::post().to(handlers::chunk_handler::get_chunks_by_ids))
                                ).service(
                                    web::resource("/tracking")
                                        .route(web::post().to(handlers::chunk_handler::get_chunks_by_tracking_ids))
                                )
                                .service(
                                    web::resource("/scroll")
                                        .route(web::post().to(handlers::chunk_handler::scroll_dataset_chunks))
                                )
                        )
                        .service(
                            web::scope("/dataset")
                                .service(
                                    web::resource("")
                                        .route(
                                            web::post().to(handlers::dataset_handler::create_dataset),
                                        )
                                        .route(web::put().to(handlers::dataset_handler::update_dataset))
                                )
                                .service(
                                    web::resource("/organization/{organization_id}").route(
                                        web::get().to(
                                            handlers::dataset_handler::get_datasets_from_organization,
                                        ),
                                    ),
                                )
                                .service(
                                    web::resource("/usage/{dataset_id}")
                                        .route(web::get().to(handlers::dataset_handler::get_usage_by_dataset_id)),
                                )
                                .service(
                                    web::resource("/{dataset_id}")
                                        .route(web::get().to(handlers::dataset_handler::get_dataset))
                                        .route(
                                            web::delete().to(handlers::dataset_handler::delete_dataset),
                                        )
                                )
                                .service(
                                    web::resource("/clear/{dataset_id}")
                                        .route(web::put().to(handlers::dataset_handler::clear_dataset)),
                                )
                                .service(
                                    web::resource("/tracking_id/{tracking_id}")
                                        .route(
                                            web::get().to(handlers::dataset_handler::get_dataset_by_tracking_id),
                                        )
                                        .route(
                                            web::delete().to(handlers::dataset_handler::delete_dataset_by_tracking_id),
                                        ),
                                )
                                .service(
                                    web::resource("/groups/{dataset_id}/{page}").route(web::get().to(
                                        handlers::group_handler::get_groups_for_dataset,
                                    )),
                                )
                                .service(web::resource("/files/{dataset_id}/{page}").route(
                                    web::get().to(handlers::file_handler::get_dataset_files_handler),
                                )),
                        )
                        .service(
                            web::scope("/auth")
                                .service(
                                    web::resource("")
                                        .route(web::get().to(handlers::auth_handler::login))
                                        .route(web::delete().to(handlers::auth_handler::logout)),
                                )
                                .service(
                                    web::resource("/me")
                                        .route(web::get().to(handlers::auth_handler::get_me)),
                                )
                                .service(
                                    web::resource("/callback")
                                        .route(web::get().to(handlers::auth_handler::callback)),
                                ),
                        )
                        .service(
                            web::resource("/topic")
                                .route(web::post().to(handlers::topic_handler::create_topic))
                                .route(web::put().to(handlers::topic_handler::update_topic)),
                        )
                        .service(
                            web::resource("/topic/{topic_id}")
                                .route(web::delete().to(handlers::topic_handler::delete_topic)),
                        )
                        .service(
                            web::resource("/topic/owner/{user_id}")
                                .route(web::get().to(handlers::topic_handler::get_all_topics_for_owner_id)),
                        )
                        .service(
                            web::resource("/message")
                                .route(
                                    web::post().to(
                                        handlers::message_handler::create_message,
                                    ),
                                )
                                .route(web::put().to(handlers::message_handler::edit_message))
                                .route(
                                    web::patch()
                                        .to(handlers::message_handler::regenerate_message_patch),
                                )
                                .route(
                                    web::delete()
                                        .to(handlers::message_handler::regenerate_message),
                                ),
                        )
                        .service(
                            web::resource("/messages/{messages_topic_id}").route(
                                web::get().to(handlers::message_handler::get_all_topic_messages),
                            ),
                        )
                        .service(
                            web::scope("/chunk")
                                .service(
                                    web::resource("")
                                        .route(web::post().to(handlers::chunk_handler::create_chunk))
                                        .route(web::put().to(handlers::chunk_handler::update_chunk)),
                                )
                                .service(web::resource("/recommend").route(
                                    web::post().to(handlers::chunk_handler::get_recommended_chunks),
                                )
                                .wrap(Compress::default())
                            )
                                .service(
                                    web::resource("/autocomplete")
                                        .wrap(Compress::default())
                                        .route(web::post().to(handlers::chunk_handler::autocomplete)),
                                )
                                .service(
                                    web::resource("/search")
                                        .wrap(Compress::default())
                                        .route(web::post().to(handlers::chunk_handler::search_chunks)),
                                )
                                .service(
                                    web::resource("/count")
                                        .route(web::post().to(handlers::chunk_handler::count_chunks)),
                                )
                                .service(web::resource("/suggestions").route(
                                    web::post().to(
                                        handlers::message_handler::get_suggested_queries,
                                    ),
                                ))
                                .service(web::resource("/generate").route(
                                    web::post().to(handlers::chunk_handler::generate_off_chunks),
                                ))
                                .service(web::resource("/tracking_id/update").route(
                                    web::put().to(handlers::chunk_handler::update_chunk_by_tracking_id),
                                ))
                                .service(
                                    web::resource("/{id}")
                                        .route(web::get().to(handlers::chunk_handler::get_chunk_by_id))
                                        .route(web::delete().to(handlers::chunk_handler::delete_chunk)),
                                )
                                .service(
                                    web::resource("/tracking_id/{tracking_id}")
                                        .route(
                                            web::get()
                                                .to(handlers::chunk_handler::get_chunk_by_tracking_id),
                                        )
                                        .route(
                                            web::delete().to(
                                                handlers::chunk_handler::delete_chunk_by_tracking_id,
                                            ),
                                        ),
                                )
                        )
                        .service(
                            web::scope("/user")
                                .service(
                                    web::resource("")
                                        .route(web::put().to(handlers::user_handler::update_user)),
                                )
                                .service(
                                    web::resource("/api_key")
                                        .route(web::post().to(handlers::user_handler::set_user_api_key))
                                        .route(web::get().to(handlers::user_handler::get_user_api_keys))
                                )
                                .service(
                                    web::resource("/api_key/{api_key_id}")
                                        .route(
                                            web::delete().to(handlers::user_handler::delete_user_api_key),
                                        ),
                                )
                        )
                        .service(
                            web::scope("/chunk_group")
                                .service(
                                    web::resource("")
                                        .route(
                                            web::post().to(handlers::group_handler::create_chunk_group),
                                        )
                                        .route(
                                            web::put().to(handlers::group_handler::update_chunk_group),
                                        ),
                                    )
                                .service(web::resource("/chunks").route(
                                    web::post().to(handlers::group_handler::get_groups_for_chunks),
                                ))
                                .service(
                                    web::resource("/search")
                                        .route(web::post().to(handlers::group_handler::search_within_group))
                                        .wrap(Compress::default())
,
                                )
                                .service(
                                    web::resource("/group_oriented_search").route(
                                        web::post().to(handlers::group_handler::search_over_groups),
                                    )
                                    .wrap(Compress::default())
                                )
                                .service(
                                    web::resource("/recommend").route(
                                        web::post().to(handlers::group_handler::get_recommended_groups),
                                    )                                        .wrap(Compress::default())
,
                                )
                                .service(
                                    web::resource("/chunk/{chunk_group_id}")
                                        .route(
                                            web::delete()
                                                .to(handlers::group_handler::remove_chunk_from_group),
                                        ).route(web::post().to(handlers::group_handler::add_chunk_to_group))
                                )
                                .service(
                                    web::scope("/tracking_id/{tracking_id}")
                                        .service(
                                            web::resource("")
                                                .route(
                                                    web::get().to(
                                                        handlers::group_handler::get_group_by_tracking_id,
                                                    ),
                                                )
                                                .route(
                                                    web::post().to(
                                                        handlers::group_handler::add_chunk_to_group_by_tracking_id
                                                    )
                                                )
                                                .route(
                                                    web::delete().to(
                                                        handlers::group_handler::delete_group_by_tracking_id,
                                                    )
                                                )
                                                .route(
                                                    web::put().to(handlers::group_handler::update_group_by_tracking_id),
                                                )
                                        ).service(
                                            web::resource("/{page}").route(
                                                web::get().to(
                                                    handlers::group_handler::get_chunks_in_group_by_tracking_id,
                                                ),
                                            ),
                                        ),
                                )
                                .service(
                                    web::scope("/{group_id}")
                                        .service(
                                            web::resource("")
                                                .route(web::get().to(handlers::group_handler::get_chunk_group))
                                                .route(web::delete().to(handlers::group_handler::delete_chunk_group)),
                                        )
                                        .service(
                                            web::resource("/{page}")
                                                .route(web::get().to(handlers::group_handler::get_chunks_in_group)),
                                        )
                                )

                        )
                        .service(
                            web::scope("/file")
                                .service(
                                    web::resource("").route(
                                        web::post().to(handlers::file_handler::upload_file_handler),
                                    ),
                                )
                                .service(
                                    web::resource("/{file_id}")
                                        .route(web::get().to(handlers::file_handler::get_file_handler))
                                        .route(
                                            web::delete()
                                                .to(handlers::file_handler::delete_file_handler),
                                        ),
                                )
                                .service(
                                    web::resource("/get_signed_url/{file_name}")
                                        .route(web::get().to(handlers::file_handler::get_signed_url)),
                                )
                        )
                        .service(
                            web::scope("/events").service(
                                web::resource("")
                                    .route(web::post().to(handlers::event_handler::get_events)),
                            ),
                        )
                        .service(
                            web::resource("/health")
                                .route(web::get().to(handlers::auth_handler::health_check)),
                        )
                        .service(
                            web::scope("/organization")
                                .service(
                                    web::resource("/usage/{organization_id}")
                                        .route(web::get().to(
                                            handlers::organization_handler::get_organization_usage,
                                        )),
                                )
                                .service(
                                    web::resource("/users/{organization_id}")
                                        .route(web::get().to(
                                            handlers::organization_handler::get_organization_users,
                                        )),
                                )
                                .service(
                                    web::resource("/update_dataset_configs")
                                        .route(web::post().to(handlers::organization_handler::update_all_org_dataset_configs)),
                                )
                                .service(
                                    web::resource("/{organization_id}/user/{user_id}")
                                        .route(web::delete().to(handlers::organization_handler::remove_user_from_org)),
                                )
                                .service(
                                    web::resource("/{organization_id}")
                                        .route(
                                            web::get().to(
                                                handlers::organization_handler::get_organization,
                                            ),
                                        )
                                        .route(web::delete().to(
                                            handlers::organization_handler::delete_organization,
                                        )),
                                )
                                .service(
                                    web::resource("")
                                        .route(
                                            web::post().to(
                                                handlers::organization_handler::create_organization,
                                            ),
                                        )
                                        .route(
                                            web::put().to(
                                                handlers::organization_handler::update_organization,
                                            ),
                                        ),
                                ),
                        )
                        .service(
                            web::scope("/invitation")
                                .service(
                                    web::resource("")
                                        .route(web::post().to(handlers::invitation_handler::post_invitation)),
                                )
                                .service(
                                    web::resource("/{organization_id}")
                                        .route(web::get().to(handlers::invitation_handler::get_invitations))
                                        .route(web::delete().to(handlers::invitation_handler::delete_invitation)),
                                ),
                            )
                        .service(
                            web::scope("/stripe")
                                .service(
                                    web::resource("/webhook")
                                        .route(web::post().to(handlers::stripe_handler::webhook)),
                                )
                                .service(web::resource("/subscription/{subscription_id}").route(
                                    web::delete().to(handlers::stripe_handler::cancel_subscription),
                                ))
                                .service(
                                    web::resource("/subscription_plan/{subscription_id}/{plan_id}")
                                        .route(
                                            web::patch()
                                                .to(handlers::stripe_handler::update_subscription_plan),
                                        ),
                                )
                                .service(
                                    web::resource("/payment_link/{plan_id}/{organization_id}").route(
                                        web::get().to(handlers::stripe_handler::direct_to_payment_link),
                                    ),
                                )
                                .service(
                                    web::resource("/plans")
                                        .route(web::get().to(handlers::stripe_handler::get_all_plans)),
                                )
                                .service(
                                    web::resource("/invoices/{organization_id}")
                                        .route(web::get().to(handlers::stripe_handler::get_all_invoices)),
                                )
                                .service(
                                    web::resource("/checkout/setup/{organization_id}")
                                        .route(web::post().to(handlers::stripe_handler::create_setup_checkout_session)),
                                ),
                        )
                        .service(
                            web::scope("/analytics")
                            .service(
                                web::resource("/search")
                                .route(web::post().to(handlers::analytics_handler::get_search_analytics)),
                            )
                            .service(
                                web::resource("/search/clusters")
                                .route(web::post().to(handlers::analytics_handler::get_cluster_analytics)),
                            )
                            .service(
                                web::resource("/rag")
                                .route(web::post().to(handlers::analytics_handler::get_rag_analytics)),
                            )
                            .service(
                                web::resource("/recommendations")
                                .route(web::post().to(handlers::analytics_handler::get_recommendation_analytics)),
                            )
                            .service(
                                web::resource("/ctr")
                                .route(web::put().to(handlers::analytics_handler::send_ctr_data))
                                .route(web::post().to(handlers::analytics_handler::get_ctr_analytics)),
                            )
                        ),
                )
        })
        .bind(("0.0.0.0", 8090))?
        .run()
        .await

    })?;

    Ok(())
}
