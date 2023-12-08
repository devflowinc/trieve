#[macro_use]
extern crate diesel;
use crate::{
    errors::ServiceError,
    operators::tantivy_operator::TantivyIndexMap,
};
use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::RedisSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{Key, SameSite},
    middleware,
    web::{self, PayloadConfig},
    App, HttpServer,
};
use diesel::{prelude::*, r2d2};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use pyo3::{types::PyDict, Py, PyAny, Python};
use tokio::sync::{RwLock, Semaphore};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

mod data;
mod errors;
mod handlers;
mod operators;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
pub const SECONDS_IN_MINUTE: u64 = 60;
pub const SECONDS_IN_HOUR: u64 = 60 * SECONDS_IN_MINUTE;
pub const SECONDS_IN_DAY: u64 = 24 * SECONDS_IN_HOUR;

fn run_migrations(conn: &mut impl MigrationHarness<diesel::pg::Pg>) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

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
#[derive(Clone)]
pub struct CrossEncoder {
    tokenizer: Py<PyAny>,
    model: Py<PyAny>,
}

fn initalize_cross_encoder() -> CrossEncoder {
    let cross_encoder = Python::with_gil(|py| {
        let transformers = py.import("transformers").unwrap();

        let tokenizer: Py<PyAny> = transformers
            .getattr("AutoTokenizer")
            .map_err(|e| ServiceError::BadRequest(format!("Could not get tokenizer: {}", e)))?
            .call_method1::<&str, (&str,)>(
                "from_pretrained",
                ("cross-encoder/ms-marco-MiniLM-L-4-v2",),
            )
            .map_err(|e| ServiceError::BadRequest(format!("Could not load tokenizer: {}", e)))?
            .into();

        let onnxruntime = py.import("optimum.onnxruntime").map_err(|e| {
            ServiceError::BadRequest(format!("Could not import onnxruntime: {}", e))
        })?;
        let model_kwargs = PyDict::new(py);

        model_kwargs
            .set_item("from_transformers", true)
            .map_err(|e| {
                ServiceError::BadRequest(format!("Could not set from_transformers: {}", e))
            })?;
        model_kwargs
            .set_item("force_download", false)
            .map_err(|e| {
                ServiceError::BadRequest(format!("Could not set force_download: {}", e))
            })?;

        let model: Py<PyAny> = onnxruntime
            .getattr("ORTModelForSequenceClassification")
            .map_err(|e| ServiceError::BadRequest(format!("Could not get model: {}", e)))?
            .call_method::<&str, (&str,)>(
                "from_pretrained",
                ("cross-encoder/ms-marco-MiniLM-L-4-v2",),
                Some(model_kwargs),
            )
            .map_err(|e| ServiceError::BadRequest(format!("Could not load model: {}", e)))?
            .into();
        Ok::<CrossEncoder, ServiceError>(CrossEncoder { tokenizer, model })
    });
    cross_encoder.unwrap()
}

pub struct AppMutexStore {
    pub embedding_semaphore: Option<Semaphore>,
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    #[derive(OpenApi)]
    #[openapi(
        info(description = "Arguflow REST API OpenAPI Documentation"),
        paths(
            handlers::invitation_handler::post_invitation,
            handlers::register_handler::register_user,
            handlers::auth_handler::login,
            handlers::auth_handler::logout,
            handlers::auth_handler::get_me,
            handlers::password_reset_handler::reset_user_password_handler,
            handlers::password_reset_handler::send_password_reset_email_handler,
            handlers::topic_handler::create_topic,
            handlers::topic_handler::delete_topic,
            handlers::topic_handler::update_topic,
            handlers::topic_handler::get_all_topics,
            handlers::message_handler::create_message_completion_handler,
            handlers::message_handler::get_all_topic_messages,
            handlers::message_handler::edit_message_handler,
            handlers::message_handler::regenerate_message_handler,
            handlers::card_handler::create_card,
            handlers::card_handler::update_card,
            handlers::card_handler::get_recommended_cards,
            handlers::message_handler::create_suggested_queries_handler,
            handlers::card_handler::update_card_by_tracking_id,
            handlers::card_handler::search_card,
            handlers::card_handler::generate_off_cards,
            handlers::card_handler::get_card_by_tracking_id,
            handlers::card_handler::delete_card_by_tracking_id,
            handlers::card_handler::get_card_by_id,
            handlers::user_handler::update_user,
            handlers::user_handler::set_user_api_key,
            handlers::user_handler::get_user_with_cards_by_id,
            handlers::file_handler::get_user_files_handler,
            handlers::collection_handler::get_specific_user_card_collections,
            handlers::collection_handler::create_card_collection,
            handlers::collection_handler::delete_card_collection,
            handlers::collection_handler::update_card_collection,
            handlers::collection_handler::add_bookmark,
            handlers::collection_handler::delete_bookmark,
            handlers::collection_handler::get_logged_in_user_card_collections,
            handlers::collection_handler::get_all_bookmarks,
            handlers::file_handler::update_file_handler,
            handlers::file_handler::upload_file_handler,
            handlers::file_handler::get_file_handler,
            handlers::file_handler::delete_file_handler,
            handlers::file_handler::get_image_file,
            handlers::notification_handler::mark_notification_as_read,
            handlers::notification_handler::get_notifications,
            handlers::notification_handler::mark_all_notifications_as_read,
            handlers::auth_handler::health_check,
            handlers::organization_handler::get_organization_by_id,
            handlers::organization_handler::delete_organization_by_id,
            handlers::organization_handler::update_organization,
            handlers::organization_handler::create_organization,
        ),
        components(
            schemas(
                handlers::invitation_handler::InvitationResponse,
                handlers::invitation_handler::InvitationData,
                handlers::register_handler::SetPasswordData,
                handlers::auth_handler::AuthData,
                handlers::password_reset_handler::PasswordResetData,
                handlers::password_reset_handler::PasswordResetEmailData,
                handlers::topic_handler::CreateTopicData,
                handlers::topic_handler::DeleteTopicData,
                handlers::topic_handler::UpdateTopicData,
                handlers::message_handler::CreateMessageData,
                handlers::message_handler::RegenerateMessageData,
                handlers::message_handler::EditMessageData,
                handlers::message_handler::SuggestedQueriesRequest,
                handlers::message_handler::SuggestedQueriesResponse,
                handlers::card_handler::CreateCardData,
                handlers::card_handler::ReturnCreatedCard,
                handlers::card_handler::UpdateCardData,
                handlers::card_handler::RecommendCardsRequest,
                handlers::card_handler::UpdateCardByTrackingIdData,
                handlers::card_handler::SearchCardQueryResponseBody,
                handlers::card_handler::GenerateCardsRequest,
                handlers::card_handler::SearchCardData,
                handlers::card_handler::ScoreCardDTO,
                handlers::card_handler::SearchCollectionsData,
                handlers::card_handler::SearchCollectionsResult,
                handlers::user_handler::UpdateUserData,
                handlers::user_handler::GetUserWithCardsData,
                handlers::user_handler::SetUserApiKeyResponse,
                handlers::collection_handler::CollectionData,
                handlers::collection_handler::UserCollectionQuery,
                handlers::collection_handler::CreateCardCollectionData,
                handlers::collection_handler::DeleteCollectionData,
                handlers::collection_handler::UpdateCardCollectionData,
                handlers::collection_handler::AddCardToCollectionData,
                handlers::collection_handler::GetCollectionsForCardsData,
                handlers::collection_handler::RemoveBookmarkData,
                handlers::collection_handler::GenerateOffCollectionData,
                handlers::collection_handler::GetAllBookmarksData,
                handlers::collection_handler::BookmarkCards,
                handlers::collection_handler::BookmarkData,
                operators::collection_operator::BookmarkCollectionResult,
                handlers::file_handler::UploadFileData,
                handlers::file_handler::UploadFileResult,
                handlers::file_handler::UpdateFileData,
                handlers::notification_handler::NotificationId,
                handlers::notification_handler::Notification,
                handlers::organization_handler::CreateOrganizationData,
                handlers::organization_handler::UpdateOrganizationData,
                operators::notification_operator::NotificationReturn,
                data::models::SlimUser,
                data::models::UserDTO,
                data::models::Topic,
                data::models::Message,
                data::models::CardMetadata,
                data::models::CardMetadataWithFileData,
                data::models::ChatMessageProxy,
                data::models::UserDTOWithCards,
                data::models::File,
                data::models::CardCollectionAndFile,
                data::models::CardCollection,
                data::models::FileDTO,
                data::models::FileUploadCompletedNotificationWithName,
                data::models::Organization,
                errors::DefaultError,
            )
        ),
        tags(
            (name = "invitation", description = "Invitations for new users endpoint"),
            (name = "register", description = "Register new users endpoint"),
            (name = "auth", description = "Authentication endpoint"),
            (name = "password", description = "Password reset endpoint"),
            (name = "topic", description = "Topic chat endpoint"),
            (name = "message", description = "Message chat endpoint"),
            (name = "card", description = "Card endpoint"),
            (name = "user", description = "User endpoint"),
            (name = "card_collection", description = "Card collection endpoint"),
            (name = "file", description = "File endpoint"),
            (name = "notifications", description = "Notifications endpoint"),
            (name = "health", description = "Health check endpoint"),
        ),
    )]
    struct ApiDoc;

    dotenvy::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    if std::env::var("ALERT_EMAIL").is_err() {
        log::warn!("ALERT_EMAIL not set, this might be useful during health checks");
    }

    let database_url = get_env!("DATABASE_URL", "DATABASE_URL should be set");
    let redis_url = get_env!("REDIS_URL", "REDIS_URL should be set");

    // create db connection pool
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    let pool: data::models::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let cross_encoder = initalize_cross_encoder();

    let redis_store = RedisSessionStore::new(redis_url).await.unwrap();

    let tantivy_index = web::Data::new(RwLock::new(TantivyIndexMap::new()));

    run_migrations(&mut pool.get().unwrap());

    log::info!("starting HTTP server at http://localhost:8090");

    let app_mutex_store = web::Data::new(AppMutexStore {
        embedding_semaphore: std::env::var("EMBEDDING_SEMAPHORE_SIZE")
            .map(|size| match size.parse::<usize>() {
                Ok(size) => Some(Semaphore::new(size)),
                Err(_) => None,
            })
            .unwrap_or(None),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(PayloadConfig::new(134200000))
            .app_data( web::JsonConfig::default().limit(134200000))
            .app_data(web::Data::new(pool.clone()))
            .app_data(app_mutex_store.clone())
            .app_data(tantivy_index.clone())
            .wrap(
                IdentityMiddleware::builder()
                    .login_deadline(Some(std::time::Duration::from_secs(SECONDS_IN_DAY)))
                    .visit_deadline(Some(std::time::Duration::from_secs(SECONDS_IN_DAY)))
                    .build(),
            )
            .wrap(Cors::permissive())
            .wrap(
                SessionMiddleware::builder(
                    redis_store.clone(),
                    Key::from(handlers::register_handler::SECRET_KEY.as_bytes()),
                )
                .session_lifecycle(
                    PersistentSession::default().session_ttl(time::Duration::days(1)),
                )
                .cookie_name("vault".to_owned())
                .cookie_same_site(if std::env::var("COOKIE_SECURE").unwrap_or("false".to_owned()) == "true" {
                    SameSite::None
                } else {
                    SameSite::Lax
                })
                .cookie_secure(std::env::var("COOKIE_SECURE").unwrap_or("false".to_owned()) == "true")
                .cookie_path("/".to_owned())
                .build(),
            )
            // enable logger
            .wrap(middleware::Logger::default())
            .service(Redoc::with_url("/redoc", ApiDoc::openapi()))
            // everything under '/api/' route
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/dataset").route(web::post().to(handlers::dataset_handler::create_dataset)),
                    )
                    .service(
                        web::resource("/invitation")
                            .route(web::post().to(handlers::invitation_handler::post_invitation)),
                    )
                    .service(
                        web::resource("/register/{invitation_id}")
                            .route(web::post().to(handlers::register_handler::register_user)),
                    )
                    .service(
                        web::resource("/auth")
                            .route(web::post().to(handlers::auth_handler::login))
                            .route(web::delete().to(handlers::auth_handler::logout))
                            .route(web::get().to(handlers::auth_handler::get_me)),
                    )
                    .service(
                        web::scope("/password")
                            .service(
                                web::resource("").route(
                                    web::post()
                                        .to(handlers::password_reset_handler::reset_user_password_handler),
                                )
                            )
                            .service(web::resource("/{email}").route(
                                web::get().to(
                                    handlers::password_reset_handler::send_password_reset_email_handler,
                                ),
                            ),
                            ),
                    )
                    .service(
                        web::resource("/topic")
                            .route(web::post().to(handlers::topic_handler::create_topic))
                            .route(web::delete().to(handlers::topic_handler::delete_topic))
                            .route(web::put().to(handlers::topic_handler::update_topic))
                            .route(web::get().to(handlers::topic_handler::get_all_topics)),
                    )
                    .service(
                        web::resource("/message")
                            .route(
                                web::post().to(
                                    handlers::message_handler::create_message_completion_handler,
                                ),
                            )
                            .route(web::put().to(handlers::message_handler::edit_message_handler))
                            .route(
                                web::delete()
                                    .to(handlers::message_handler::regenerate_message_handler),
                            ),
                    )
                    .service(
                        web::resource("/messages/{messages_topic_id}").route(
                            web::get().to(handlers::message_handler::get_all_topic_messages),
                        ),
                    )
                    .service(
                        web::scope("/card")
                            .service(
                                web::resource("")
                                    .route(web::post().to(handlers::card_handler::create_card)),
                            )
                            .service(
                                web::resource("/recommend").route(
                                    web::post().to(handlers::card_handler::get_recommended_cards),
                                ),
                            )
                            .service(
                                web::resource("/update")
                                    .route(web::put().to(handlers::card_handler::update_card)),
                            )
                            .service(
                                web::resource("/search")
                                    .route(web::post().to(handlers::card_handler::search_card)),
                            )
                            .service(
                                web::resource("/gen_suggestions")
                                    .route(web::post().to(handlers::message_handler::create_suggested_queries_handler)),
                            )
                            .service(
                                web::resource("/search/{page}")
                                    .app_data(web::Data::new(cross_encoder.clone()))
                                    .route(web::post().to(handlers::card_handler::search_card)),
                            )
                            .service(
                                web::resource("/generate")
                                .route(web::post().to(handlers::card_handler::generate_off_cards)),
                            )
                            .service(
                                web::resource("/tracking_id/update")
                                    .route(web::put().to(handlers::card_handler::update_card_by_tracking_id)),
                            )
                            .service(
                                web::resource("/tracking_id/{tracking_id}")
                                    .route(web::get().to(handlers::card_handler::get_card_by_tracking_id))
                                    .route(web::delete().to(handlers::card_handler::delete_card_by_tracking_id))
                            )
                            .service(
                                web::resource("/{card_id}")
                                    .route(web::get().to(handlers::card_handler::get_card_by_id))
                                    .route(web::delete().to(handlers::card_handler::delete_card)),
                            )
                    ).service(
                        web::scope("/user")
                            .service(web::resource("")
                                .route(web::put().to(handlers::user_handler::update_user)),
                            )
                            .service(web::resource("/set_api_key")
                                .route(web::get().to(handlers::user_handler::set_user_api_key)),
                            )
                            .service(web::resource("/{user_id}/{page}")
                                .route(web::get().to(handlers::user_handler::get_user_with_cards_by_id)),
                            )
                            .service(
                                web::resource("/files/{user_id}")
                                    .route(web::get().to(handlers::file_handler::get_user_files_handler)),
                            )
                            .service(
                                web::resource("/collections/{user_id}/{page}").route(
                                    web::get().to(
                                        handlers::collection_handler::get_specific_user_card_collections,
                                    ),
                                ),
                            ),
                    )
                    .service(
                        web::scope("/card_collection")
                            .service(
                                web::resource("")
                                    .route(
                                        web::post().to(
                                            handlers::collection_handler::create_card_collection,
                                        ),
                                    )
                                    .route(
                                        web::delete().to(
                                            handlers::collection_handler::delete_card_collection,
                                        ),
                                    )
                                    .route(
                                        web::put().to(
                                            handlers::collection_handler::update_card_collection,
                                        ),
                                    ),
                            )
                            .service(
                                web::resource("/generate").route(
                                    web::post().to(
                                        handlers::collection_handler::generate_off_collection,
                                    ),
                                ),
                            )
                            .service(
                                web::resource("/bookmark").route(
                                    web::post().to(
                                        handlers::collection_handler::get_collections_card_is_in,
                                    ),
                                ),
                            )
                            .service(
                                web::resource("/{page_or_card_collection_id}")
                                    .route(
                                        web::post().to(handlers::collection_handler::add_bookmark),
                                    )
                                    .route(
                                        web::delete()
                                            .to(handlers::collection_handler::delete_bookmark),
                                    ).route(
                                        web::get()
                                            .to(handlers::collection_handler::get_logged_in_user_card_collections)),
                            )
                            .service(
                                web::resource("/search/{page}")                                    
                                .route(
                                    web::post().to(handlers::card_handler::search_collections),
                                ),
                            )
                            .service(web::resource("/{collection_id}/{page}").route(
                                web::get().to(handlers::collection_handler::get_all_bookmarks),
                            )),
                    )
                    .service(
                web::scope("/file")
                            .service(
                                web::resource("")
                                    .route(web::put().to(handlers::file_handler::update_file_handler))
                                    .route(web::post().to(handlers::file_handler::upload_file_handler)),
                            )
                            .service(
                                web::resource("/{file_id}")
                                    .route(web::get().to(handlers::file_handler::get_file_handler))
                                    .route(web::delete().to(handlers::file_handler::delete_file_handler)),
                            ),
                    )
                    .service(
                        web::resource("/image/{file_name}").route(
                            web::get().to(handlers::file_handler::get_image_file),
                        ),
                    )
                    .service(
                        web::resource("/pdf_from_range/{file_start}/{file_end}/{prefix}/{file_name}/{ocr}").route(
                            web::get().to(handlers::file_handler::get_pdf_from_range),
                        ),
                    )
                    .service(
                        web::scope("/notifications")
                            .service(web::resource("").route(
                                web::put().to(handlers::notification_handler::mark_notification_as_read),
                            ))
                            .service(
                                web::resource("/{page}").route(
                                    web::get().to(handlers::notification_handler::get_notifications),
                                ),
                            ),
                    )
                    .service(
                        web::resource("/notifications_readall")
                            .route(web::put().to(
                                handlers::notification_handler::mark_all_notifications_as_read,
                            )),
                    )
                    .service(
                        web::resource("/health").route(web::get().to(handlers::auth_handler::health_check)),
                    )
                .service(
                    web::scope("/organization")
                    .service(
                        web::resource("/{organization_id}")
                            .route(web::get().to(handlers::organization_handler::get_organization_by_id))
                            .route(web::delete().to(handlers::organization_handler::delete_organization_by_id))
                    )
                    .service(
                        web::resource("")
                            .route(web::post().to(handlers::organization_handler::create_organization))
                            .route(web::put().to(handlers::organization_handler::update_organization))
                    )
                )
            )
    })
    .bind(("0.0.0.0", 8090))?
    .run()
    .await
}
