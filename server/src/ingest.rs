use actix_web::{HttpServer, App, web};

use crate::handlers;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting HTTP server at http://localhost:9090");
    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/ingest")
                            .route(web::get().to(handlers::ingest_handler::ingest))
                    )
            )
    })
    .bind(("0.0.0.0", 9090))?
    .run()
    .await
}
