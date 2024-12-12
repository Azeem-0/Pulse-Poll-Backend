use actix_web::{
    get, post,
    web::{self, Data, Path},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use chrono::Utc;
use nanoid::nanoid;

use crate::{
    db::mongodb_repository::MongoDB,
    middlewares::jwt_middleware::jwt_middleware,
    models::poll_model::{OptionItem, Poll},
    utils::{
        jwt_token_generation::Claims,
        types::{PollCreation, UserNameRequest, VoteOption},
    },
};

#[get("/")]
async fn get_all_polls(db: Data<MongoDB>) -> impl Responder {
    let polls = match db.poll_repository.get_all_polls().await {
        Ok(polls) => polls,
        Err(err) => {
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    HttpResponse::Ok().json(polls)
}

// #[post("/polls")]
async fn create_new_poll(
    // req: HttpRequest,
    db: Data<MongoDB>,
    data: web::Json<PollCreation>,
) -> impl Responder {
    let poll_id = nanoid!(10);

    // let user = req.extensions().get::<Claims>().cloned().unwrap();

    let title = data.title.clone();

    let username = data.username.clone();

    let options = data.options.clone();

    let options = options
        .into_iter()
        .map(|text| OptionItem {
            option_id: nanoid!(10),
            text: text.clone(),
            votes: 0,
        })
        .collect();

    let now = Utc::now();

    let poll = Poll {
        poll_id,
        username,
        title,
        options,
        is_active: true,
        voters: vec![],
        created_at: now,
        updated_at: now,
    };

    if let Err(err) = db.poll_repository.create_poll(poll).await {
        return HttpResponse::InternalServerError().body(err.to_string());
    }

    HttpResponse::Ok().body("New poll created successfully.")
}

#[get("/polls/{id}")]
async fn get_poll_by_id(db: Data<MongoDB>, id: Path<String>) -> impl Responder {
    let poll_option = match db.poll_repository.get_poll_by_id(&id).await {
        Ok(poll_option) => poll_option,
        Err(err) => {
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    let poll = match poll_option {
        Some(poll) => poll,
        None => {
            return HttpResponse::Ok().body("No poll found with given id.");
        }
    };

    HttpResponse::Ok().json(poll)
}

// #[post("/polls/{id}/vote")]
async fn cast_vote_to_poll(
    db: Data<MongoDB>,
    id: Path<String>,
    data: web::Json<VoteOption>,
) -> impl Responder {
    let username = &data.username;

    let option_id = &data.option_id;

    let user_voted = match db
        .poll_repository
        .check_user_vote_in_poll(username, &id)
        .await
    {
        Ok(res) => res,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Error accessing poll");
        }
    };

    if user_voted {
        if let Err(err) = db
            .poll_repository
            .change_vote_in_poll_by_id(&id, option_id, username)
            .await
        {
            return HttpResponse::InternalServerError()
                .body(format!("Error changing vote: {}", err));
        }
        return HttpResponse::Ok().body("Successfully updated your vote.");
    } else {
        if let Err(err) = db
            .poll_repository
            .cast_vote_to_poll_by_id(&id, option_id, username)
            .await
        {
            return HttpResponse::InternalServerError()
                .body(format!("Error voting to the poll {}", err));
        }
    }

    HttpResponse::Ok().body("Successfully voted to the poll.")
}

// #[post("/polls/{id}/close")]
async fn close_poll_by_id(
    db: Data<MongoDB>,
    id: Path<String>,
    data: web::Json<UserNameRequest>,
) -> impl Responder {
    let username = &data.username;
    if let Err(err) = db.poll_repository.close_poll_by_id(&id, username).await {
        return HttpResponse::InternalServerError().body(format!("Error closing poll : {}", err));
    }

    HttpResponse::Ok().body("Closed poll successfully.")
}

// #[post("/polls/{id}/reset")]
async fn reset_votes_by_id(
    db: Data<MongoDB>,
    id: Path<String>,
    data: web::Json<UserNameRequest>,
) -> impl Responder {
    let username = &data.username;

    if let Err(err) = db.poll_repository.reset_poll_by_id(&id, username).await {
        return HttpResponse::InternalServerError().body(format!("Error resetting poll : {}", err));
    }

    HttpResponse::Ok().body("Poll reset successfully.")
}

// #[get("/polls/{id}/results")]
// async fn fetch_results_by_id(db: Data<MongoDB>, id: Path<String>) -> impl Responder {
//     HttpResponse::Ok().body("No poll found with given id.")
// }

pub fn init(config: &mut web::ServiceConfig) -> () {
    // don't forget to add the middleware later..
    config
        .service(get_all_polls)
        .service(
            web::scope("/polls")
                .wrap(actix_web::middleware::from_fn(jwt_middleware))
                .route("/", web::post().to(create_new_poll))
                .route("/{id}/vote", web::post().to(cast_vote_to_poll))
                .route("/{id}/close", web::post().to(close_poll_by_id))
                .route("/{id}/reset", web::post().to(reset_votes_by_id)),
        )
        .service(get_poll_by_id);
    // .service(create_new_poll)
    // .service(cast_vote_to_poll)
    // .service(close_poll_by_id)
    // .service(reset_votes_by_id);

    // config.service(
    //     web::scope("/route")
    //         .wrap(actix_web::middleware::from_fn(jwt_middleware))
    //         .route("/protected", web::get().to(protected_route)),
    // );

    ()
}
