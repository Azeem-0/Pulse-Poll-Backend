use actix_web::{web, HttpResponse};

use crate::middlewares::jwt_middleware::jwt_middleware;

pub async fn protected_route() -> HttpResponse {
    HttpResponse::Ok().body("Protected data")
}
pub fn init(config: &mut web::ServiceConfig) -> () {
    config.service(
        web::scope("/route")
            .wrap(actix_web::middleware::from_fn(jwt_middleware))
            .route("/protected", web::get().to(protected_route)),
    );

    ()
}
