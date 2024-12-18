use actix_web::{
    get,
    web::{self, Data, Path, Query},
    HttpResponse, Responder,
};
use chrono::Utc;
use nanoid::nanoid;
use std::sync::Mutex;

use crate::{
    db::mongodb_repository::MongoDB,
    middlewares::jwt_middleware::jwt_middleware,
    models::{
        broadcaster_model::Broadcaster,
        poll_model::{OptionItem, Poll, PollQueryParams},
    },
    utils::{
        poll_results_utility::calculate_poll_results,
        types::{PollCreation, UserNameRequest, VoteOption},
    },
};

async fn get_poll_utility(db: &Data<MongoDB>, id: &str) -> Option<Poll> {
    let poll_option = match db.poll_repository.get_poll_by_id(id).await {
        Ok(poll_option) => poll_option,
        Err(_) => {
            return None;
        }
    };

    return poll_option;
}

#[utoipa::path(
    get,
    path = "/api/",
    responses(
        (status = 200, description = "Successfully fetched all polls", body = Vec<Poll>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Polls",
    operation_id = "getAllPolls"
)]
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

#[utoipa::path(
    get,
    path = "/api/polls/{id}",
    params(
        ("id" = String, Path, description = "The unique identifier of the poll")
    ),
    responses(
        (status = 200, description = "Successfully fetched poll details", body = Poll),
        (status = 404, description = "Poll not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Polls",
    operation_id = "getPollById"
)]
#[get("/polls/{id}")]
async fn get_poll_by_id(db: Data<MongoDB>, id: Path<String>) -> impl Responder {
    let poll = match get_poll_utility(&db, &id).await {
        Some(poll) => poll,
        None => {
            return HttpResponse::Ok().body("No poll found with given id.");
        }
    };

    HttpResponse::Ok().json(poll)
}

#[utoipa::path(
    get,
    path = "/api/polls/{id}/results",
    params(
        ("id" = String, Path, description = "The unique identifier of the poll"),
        ("live" = Option<bool>, Query, description = "Filter by live status of the poll"),
        ("closed" = Option<bool>, Query, description = "Filter by closed status of the poll"),
        ("creator" = Option<String>, Query, description = "Filter by creator username")
    ),
    responses(
        (status = 200, description = "Successfully fetched poll results", body = PollResults),
        (status = 400, description = "Query parameters mismatch"),
        (status = 404, description = "Poll not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Polls",
    operation_id = "getPollResults"
)]
#[get("/polls/{id}/results")]
async fn fetch_results_by_id(
    db: Data<MongoDB>,
    id: Path<String>,
    broadcaster: Data<Mutex<Broadcaster>>,
    query: Query<PollQueryParams>,
) -> impl Responder {
    let params = query.into_inner();

    let poll_id = id.into_inner();

    match db.poll_repository.get_poll_by_id(&poll_id).await {
        Ok(Some(poll)) => {
            if let Some(close) = params.closed {
                if close && !poll.is_active || !close && poll.is_active {
                    return HttpResponse::BadRequest().body(
                        "Query parameters mismatch, poll status is not specified correctly.",
                    );
                }
            }

            if let Some(live) = params.live {
                if live && !poll.is_active || !live && poll.is_active {
                    return HttpResponse::NotImplemented().body(
                        "Query parameters mismatch, poll status is not specified correctly.",
                    );
                }
            }

            if let Some(creator_id) = params.creator {
                if poll.username != creator_id {
                    return HttpResponse::Forbidden().body(
                        "Query parameters mismatch, provided creator is not owner of this poll.",
                    );
                }
            }

            let response = calculate_poll_results(&poll);
            broadcaster.lock().unwrap().send_poll_results(&response);
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().body("No poll found with the given ID."),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[utoipa::path(
    post,
    path = "/api/polls/",
    request_body = PollCreation,
    responses(
        (status = 200, description = "Poll created successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Polls",
    operation_id = "createPoll",
    security(
        ("bearerAuth" = [])
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/polls/{id}/vote",
    request_body = VoteOption,
    responses(
        (status = 200, description = "Vote cast successfully"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Poll not found"),
        (status = 409, description = "Poll is closed"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Polls",
    operation_id = "castVoteToPoll",
    security(
        ("bearerAuth" = [])
    )
)]
async fn cast_vote_to_poll(
    db: Data<MongoDB>,
    id: Path<String>,
    data: web::Json<VoteOption>,
    broadcaster: Data<Mutex<Broadcaster>>,
) -> impl Responder {
    let poll = match get_poll_utility(&db, &id).await {
        Some(poll) => poll,
        None => {
            return HttpResponse::Ok().body("Internal server error.");
        }
    };

    if poll.is_active == false {
        return HttpResponse::Conflict().body("Cannot vote to a closed poll");
    }

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
            return HttpResponse::InternalServerError().body(format!("{}", err));
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

    let poll = match get_poll_utility(&db, &id).await {
        Some(poll) => poll,
        None => {
            return HttpResponse::Ok().body(message);
        }
    };

    broadcaster.lock().unwrap().send_updated_poll(&poll);
    let response = calculate_poll_results(&poll);
    broadcaster.lock().unwrap().send_poll_results(&response);
    HttpResponse::Ok().body("Successfully voted to the poll.")
}

#[utoipa::path(
    post,
    path = "/api/polls/{id}/close",
    request_body = UserNameRequest,
    responses(
        (status = 200, description = "Poll closed successfully"),
        (status = 404, description = "Poll not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Polls",
    operation_id = "closePoll",
    security(
        ("bearerAuth" = [])
    )
)]
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

    let poll = match get_poll_utility(&db, &id).await {
        Some(poll) => poll,
        None => {
            return HttpResponse::Ok().body("No poll found with given id.");
        }
    };

    broadcaster.lock().unwrap().send_updated_poll(&poll);
    let response = calculate_poll_results(&poll);
    broadcaster.lock().unwrap().send_poll_results(&response);
    HttpResponse::Ok().body("Closed poll successfully.")
}

#[utoipa::path(
    post,
    path = "/api/polls/{id}/reset",
    request_body = UserNameRequest,
    responses(
        (status = 200, description = "Poll reset successfully"),
        (status = 404, description = "Poll not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Polls",
    operation_id = "resetPoll",
    security(
        ("bearerAuth" = [])
    )
)]
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

    let poll = match get_poll_utility(&db, &id).await {
        Some(poll) => poll,
        None => {
            return HttpResponse::Ok().body("No poll found with given id.");
        }
    };

    broadcaster.lock().unwrap().send_updated_poll(&poll);
    let response = calculate_poll_results(&poll);
    broadcaster.lock().unwrap().send_poll_results(&response);

    HttpResponse::Ok().body("Poll reset successfully.")
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
