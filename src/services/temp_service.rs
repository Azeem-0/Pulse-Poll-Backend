use actix_web::{
    get, post,
    web::{self, Data},
    HttpResponse, Responder,
};
use std::sync::Mutex;

use crate::models::broadcaster_model::Broadcaster;

pub async fn protected_route() -> HttpResponse {
    HttpResponse::Ok().body("Protected data")
}

#[get("/create-client")]
async fn create_client(broadcaster: Data<Mutex<Broadcaster>>) -> impl Responder {
    let mut broadcaster = broadcaster.lock().unwrap();
    let client = broadcaster.new_client();

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(client)
}

#[post("/send")]
async fn send_message(broadcaster: Data<Mutex<Broadcaster>>) -> impl Responder {
    broadcaster
        .lock()
        .unwrap()
        .send("This is coming from backend.");
    HttpResponse::Ok().body("Message sent")
}
pub fn init(config: &mut web::ServiceConfig) -> () {
    config.service(create_client).service(send_message);

    ()
}
