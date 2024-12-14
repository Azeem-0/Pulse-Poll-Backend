use actix_web::{
    get, post,
    web::{self, Data, Path},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use chrono::Utc;
use nanoid::nanoid;
use serde_json::json;
use std::sync::Mutex;

use crate::{
    db::mongodb_repository::MongoDB,
    middlewares::jwt_middleware::jwt_middleware,
    models::{
        broadcaster_model::Broadcaster,
        poll_model::{OptionItem, Poll},
    },
    utils::{
        jwt_token_generation::Claims,
        poll_results_utility::{calculate_poll_results, format_duration},
        types::{PollCreation, UserNameRequest, VoteOption},
    },
};

async fn get_poll_utility(db: Data<MongoDB>, id: &str) -> Option<Poll> {
    let poll_option = match db.poll_repository.get_poll_by_id(id).await {
        Ok(poll_option) => poll_option,
        Err(_) => {
            return None;
        }
    };

    return poll_option;
}

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

#[get("/polls/{id}")]
async fn get_poll_by_id(db: Data<MongoDB>, id: Path<String>) -> impl Responder {
    let poll = match get_poll_utility(db, &id).await {
        Some(poll) => poll,
        None => {
            return HttpResponse::Ok().body("No poll found with given id.");
        }
    };

    HttpResponse::Ok().json(poll)
}

// #[post("/polls")]
async fn create_new_poll(db: Data<MongoDB>, data: web::Json<PollCreation>) -> impl Responder {
    let poll_id = nanoid!(10);

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

    if let Err(err) = db.poll_repository.create_poll(&poll).await {
        return HttpResponse::InternalServerError().body(err.to_string());
    }

    HttpResponse::Ok().body("New poll created successfully.")
}

async fn cast_vote_to_poll(
    db: Data<MongoDB>,
    id: Path<String>,
    data: web::Json<VoteOption>,
    broadcaster: Data<Mutex<Broadcaster>>,
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

    let message: &str;

    if user_voted {
        if let Err(err) = db
            .poll_repository
            .change_vote_in_poll_by_id(&id, option_id, username)
            .await
        {
            return HttpResponse::InternalServerError()
                .body(format!("Error changing vote: {}", err));
        }
        message = "Successfully changed your option.";
    } else {
        if let Err(err) = db
            .poll_repository
            .cast_vote_to_poll_by_id(&id, option_id, username)
            .await
        {
            return HttpResponse::InternalServerError()
                .body(format!("Error voting to the poll {}", err));
        }
        message = "Successfully voted for the option."
    }

    let poll = match get_poll_utility(db, &id).await {
        Some(poll) => poll,
        None => {
            return HttpResponse::Ok().body(message);
        }
    };

    broadcaster.lock().unwrap().send_updated_poll(&poll);
    let response = calculate_poll_results(&poll);
    broadcaster.lock().unwrap().send_poll_results(response);
    HttpResponse::Ok().body("Successfully voted to the poll.")
}

async fn close_poll_by_id(
    db: Data<MongoDB>,
    id: Path<String>,
    data: web::Json<UserNameRequest>,
    broadcaster: Data<Mutex<Broadcaster>>,
) -> impl Responder {
    let username = &data.username;
    if let Err(err) = db.poll_repository.close_poll_by_id(&id, username).await {
        return HttpResponse::InternalServerError().body(format!("Error closing poll : {}", err));
    }

    let poll = match get_poll_utility(db, &id).await {
        Some(poll) => poll,
        None => {
            return HttpResponse::Ok().body("No poll found with given id.");
        }
    };

    broadcaster.lock().unwrap().send_updated_poll(&poll);
    let response = calculate_poll_results(&poll);
    broadcaster.lock().unwrap().send_poll_results(response);
    HttpResponse::Ok().body("Closed poll successfully.")
}

async fn reset_votes_by_id(
    db: Data<MongoDB>,
    id: Path<String>,
    data: web::Json<UserNameRequest>,
    broadcaster: Data<Mutex<Broadcaster>>,
) -> impl Responder {
    let username = &data.username;

    if let Err(err) = db.poll_repository.reset_poll_by_id(&id, username).await {
        return HttpResponse::InternalServerError().body(format!("Error resetting poll : {}", err));
    }

    let poll = match get_poll_utility(db, &id).await {
        Some(poll) => poll,
        None => {
            return HttpResponse::Ok().body("No poll found with given id.");
        }
    };

    broadcaster.lock().unwrap().send_updated_poll(&poll);
    let response = calculate_poll_results(&poll);
    broadcaster.lock().unwrap().send_poll_results(response);

    HttpResponse::Ok().body("Poll reset successfully.")
}

#[get("/polls/{id}/results")]
async fn fetch_results_by_id(db: Data<MongoDB>, id: Path<String>) -> impl Responder {
    let poll_id = id.into_inner();

    match db.poll_repository.get_poll_by_id(&poll_id).await {
        Ok(Some(poll)) => {
            let response = calculate_poll_results(&poll);
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().body("No poll found with the given ID."),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

pub fn init(config: &mut web::ServiceConfig) -> () {
    config
        .service(get_all_polls)
        .service(get_poll_by_id)
        .service(fetch_results_by_id)
        .service(
            web::scope("/polls")
                .wrap(actix_web::middleware::from_fn(jwt_middleware))
                .route("/", web::post().to(create_new_poll))
                .route("/{id}/vote", web::post().to(cast_vote_to_poll))
                .route("/{id}/close", web::post().to(close_poll_by_id))
                .route("/{id}/reset", web::post().to(reset_votes_by_id)),
        );

    ()
}
