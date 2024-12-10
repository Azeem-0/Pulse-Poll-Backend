pub mod config;
pub mod models;
pub mod services;
pub mod startup;

async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("HELLO")
}
use actix_web::{
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use services::{login_service, register_service};
use startup::startup;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (webauthn, reg_store, login_store) = startup();

    println!("Listening on: http://0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(reg_store.clone())
            .app_data(webauthn.clone())
            .app_data(login_store.clone())
            .route("/", web::get().to(hello_world))
            .service(web::scope("/api/auth/register").configure(register_service::init))
            .service(web::scope("/api/auth/login").configure(login_service::init))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
