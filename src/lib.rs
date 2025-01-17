pub mod config;
pub mod db;
pub mod middlewares;
pub mod models;
pub mod services;
pub mod startup;
pub mod utils;

use std::io;

use utils::api_docs::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use actix_cors::Cors;
use actix_web::{
    web::{self, Data},
    App, HttpResponse, HttpServer,
};

use config::config::AppConfig;
use models::broadcaster_model::Broadcaster;
use mongodb::bson::raw::Error;

use db::mongodb_repository::MongoDB;
use services::{auth_service, poll_service, socket_service};
use startup::startup;

pub async fn home_route() -> HttpResponse {
    HttpResponse::Ok().body("Hello! Welcome to the backend api of polling application.")
}

pub async fn init_db(mongo_uri: &str, database_name: &str) -> Result<Data<MongoDB>, Error> {
    let db = MongoDB::init(mongo_uri, database_name).await.unwrap();
    Ok(Data::new(db))
}

pub async fn init_server(db_data: Data<MongoDB>) -> std::io::Result<()> {
    let webauthn = startup().map_err(|err| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to initialize Webauthn: {}", err),
        )
    })?;

    let broadcaster = Broadcaster::create();

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(broadcaster.clone())
            .app_data(db_data.clone())
            .app_data(webauthn.clone())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(web::scope("/api/auth").configure(auth_service::init))
            .service(web::scope("/api/socket").configure(socket_service::init))
            .service(web::scope("/api").configure(poll_service::init))
            .route("/", web::get().to(home_route))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials(),
            )
    })
    .bind(("0.0.0.0", 3001))?
    .run()
    .await
}

pub async fn run() -> std::io::Result<()> {
    let app_config = AppConfig::default();

    let db_data: Result<Data<MongoDB>, Error> =
        init_db(&app_config.mongodb_uri, &app_config.database_name).await;

    let db_data = match db_data {
        Ok(data) => {
            println!("Successfully connected to database.");
            data
        }
        Err(_) => {
            println!("Failed to connect to the database.");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Database connection failed",
            ));
        }
    };

    init_server(db_data).await
}
