// pub mod config;
// pub mod db;
// pub mod models;
// pub mod services;
// pub mod startup;

// async fn hello_world() -> impl Responder {
//     HttpResponse::Ok().body("HELLO")
// }
// use actix_cors::Cors;
// use actix_web::{
//     web::{self, Data},
//     App, HttpResponse, HttpServer, Responder,
// };

// use mongodb::bson::raw::Error;

// use db::mongodb_repository::MongoDB;
// use services::{login_service, register_service};
// use startup::startup;

// async fn init_db() -> Result<Data<MongoDB>, Error> {
//     let db = MongoDB::init().await.unwrap();
//     Ok(Data::new(db))
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     let (webauthn, login_store) = startup();

//     let db_data: Result<Data<MongoDB>, Error> = init_db().await;

//     let db_data = match db_data {
//         Ok(data) => {
//             println!("Successfully connected to database.");
//             data
//         }
//         Err(_) => {
//             println!("Failed to connect to the database.");
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::Other,
//                 "Database connection failed",
//             ));
//         }
//     };

//     println!("Listening on: http://0.0.0.0:8080");

//     HttpServer::new(move || {
//         App::new()
//             .app_data(db_data.clone())
//             .app_data(webauthn.clone())
//             .app_data(login_store.clone())
//             .route("/", web::get().to(hello_world))
//             .service(web::scope("/api/auth/register").configure(register_service::init))
//             .service(web::scope("/api/auth/login").configure(login_service::init))
//             .wrap(
//                 Cors::default()
//                     .allow_any_origin()
//                     .allow_any_method()
//                     .allow_any_header()
//                     .max_age(3600),
//             )
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }

use polling_application_backend::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run().await
}
