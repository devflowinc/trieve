#[macro_use]
extern crate diesel;

use crate::{
    handlers::auth_handler::create_admin_account, operators::card_operator::get_qdrant_connection,
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
use qdrant_client::{
    prelude::*,
    qdrant::{VectorParams, VectorsConfig},
};

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

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
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

    let redis_store = RedisSessionStore::new(redis_url).await.unwrap();

    let qdrant_client = get_qdrant_connection().await.unwrap();
    let qdrant_collection = std::env::var("QDRANT_COLLECTION").unwrap_or("debate_cards".to_owned());
    let embedding_size = std::env::var("EMBEDDING_SIZE").unwrap_or("1536".to_owned());
    let embedding_size = embedding_size.parse::<u64>().unwrap_or(1536);
    log::info!(
        "Qdrant collection: {} size {}",
        qdrant_collection,
        embedding_size
    );
    let _ = qdrant_client
        .create_collection(&CreateCollection {
            collection_name: qdrant_collection,
            vectors_config: Some(VectorsConfig {
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                    VectorParams {
                        size: embedding_size,
                        distance: Distance::Cosine.into(),
                        hnsw_config: None,
                        quantization_config: None,
                        on_disk: None,
                    },
                )),
            }),
            ..Default::default()
        })
        .await
        .map_err(|err| {
            log::info!("Failed to create collection: {:?}", err);
        });

    run_migrations(&mut pool.get().unwrap());

    let email = std::env::var("ADMIN_USER_EMAIL");
    let password = std::env::var("ADMIN_USER_PASSWORD");

    match (email, password) {
        (Ok(email), Ok(password)) => create_admin_account(email, password, pool.clone()).await,
        (Ok(_), Err(_)) => log::warn!("ADMIN_USER_EMAIL is set, but ADMIN_USER_PASSWORD is not"),
        (Err(_), Ok(_)) => log::warn!("ADMIN_USER_PASSWORD is set, but ADMIN_USER_EMAIL is not"),
        (Err(_), Err(_)) => log::info!("No admin user is set"),
    }

    log::info!("starting HTTP server at http://localhost:8090");

    HttpServer::new(move || {
        App::new()
            .app_data(PayloadConfig::new(134200000))
            .app_data( web::JsonConfig::default().limit(134200000))
            .app_data(web::Data::new(pool.clone()))
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
            // everything under '/api/' route
            .service(
                web::scope("/api")
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
                                web::resource("/count")
                                    .route(web::get().to(handlers::card_handler::get_total_card_count)),
                            )
                            .service(
                                web::resource("/cut")
                                    .route(web::post().to(handlers::message_handler::create_cut_card_handler)),
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
                                    .route(web::post().to(handlers::card_handler::search_card)),
                            )
                            .service(
                                web::resource("/fulltextsearch/{page}")
                                    .route(web::post().to(handlers::card_handler::search_full_text_card)),
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
                                    .route(web::delete().to(handlers::card_handler::delete_card_by_tracking_id)))
                            .service(
                                web::resource("/get_rag_cards")
                                .route(web::post().to(handlers::card_handler::get_rag_cards)),
                            )
                            .service(
                                web::resource("/{card_id}")
                                    .route(web::get().to(handlers::card_handler::get_card_by_id))
                                    .route(web::delete().to(handlers::card_handler::delete_card)),
                            )
                    )
                    .service(
                        web::scope("/vote")
                            .service(
                                web::resource("")
                                    .route(web::post().to(handlers::vote_handler::create_vote)),
                            )
                            .service(
                                web::resource("/{card_metadata_id}")
                                    .route(web::delete().to(handlers::vote_handler::delete_vote)),
                            ),
                    )
                    .service(
                        web::resource("/top_users/{page}")
                            .route(web::get().to(handlers::user_handler::get_top_users)),
                    )
                    .service(
                        web::resource("/top_cards/{page}")
                            .route(web::get().to(handlers::card_handler::get_top_cards)),
                    )
                    .service(
                        web::scope("/user")
                            .service(web::resource("")
                                .route(web::put().to(handlers::user_handler::update_user)),
                            )
                            .service(web::resource("/set_api_key")
                                .route(web::get().to(handlers::user_handler::set_user_api_key)),
                            )
                            .service(web::resource("/{user_id}/{page}")
                                .route(web::get().to(handlers::user_handler::get_user_with_votes_and_cards_by_id)),
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
                                web::resource("/search/{page}").route(
                                    web::post().to(handlers::card_handler::search_collections),
                                ),
                            )
                            .service(
                                web::resource("/fulltextsearch/{page}").route(
                                    web::post()
                                        .to(handlers::card_handler::search_full_text_collections),
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
                    ),
            )
    })
    .bind(("0.0.0.0", 8090))?
    .run()
    .await
}
