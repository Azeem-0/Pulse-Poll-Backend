use std::f64::consts::E;

use actix_web::{
    post,
    web::{self, Data, Json},
    HttpResponse,
};

use crate::models::user_model::User;
use crate::{
    config::config::{AppResult, Error, RegistrationStateStore},
    db::mongodb_repository::MongoDB,
};
use log::info;
use uuid::Uuid;
use webauthn_rs::prelude::{
    CreationChallengeResponse, RegisterPublicKeyCredential, Webauthn, WebauthnError,
};

use crate::models::auth_model::RegisterRequest;

#[post("register-start")]
pub async fn register_start(
    data: web::Json<RegisterRequest>,
    webauthn: Data<Webauthn>,
    reg_store: Data<RegistrationStateStore>,
    db: Data<MongoDB>,
) -> AppResult<Json<CreationChallengeResponse>> {
    let username = &data.username;

    let user = db.user_repository.find_user(username).await.unwrap();

    if let Some(_) = user {
        return Err(Error::UserAlreadyRegistered);
    }

    eprintln!("User Name : {}", username);

    let mut session = reg_store.lock().unwrap();

    session.remove("reg_state");

    let user_unique_id = Uuid::new_v4();

    let (ccr, reg_state) = webauthn
        .start_passkey_registration(user_unique_id, &username, &username, None)
        .map_err(|e| {
            info!("challenge_register -> {:?}", e);
            Error::Unknown(e)
        })?;

    session.insert(
        "reg_state".to_string(),
        (username.clone(), user_unique_id, reg_state),
    );

    eprintln!("{:?}", session);

    Ok(Json(ccr))
}

#[post("register-finish")]
pub async fn register_finish(
    req: web::Json<RegisterPublicKeyCredential>,
    webauthn: Data<Webauthn>,
    reg_store: Data<RegistrationStateStore>,
    db: Data<MongoDB>,
) -> AppResult<HttpResponse> {
    let mut session = reg_store.lock().unwrap();

    let (username, user_unique_id, reg_state) = match &session.get("reg_state") {
        Some((username, user_unique_id, reg_state)) => {
            (username.clone(), user_unique_id.clone(), reg_state.clone())
        }
        None => return Err(Error::CorruptSession),
    };

    session.remove("reg_state");

    let sk = webauthn
        .finish_passkey_registration(&req, &reg_state)
        .map_err(|e| {
            info!("challenge_register -> {:?}", e);
            Error::BadRequest(e)
        })?;

    let user = User::init(&username, &sk);

    match db.user_repository.insert_user(&user).await {
        Ok(_) => println!("Inserted successfully"),
        Err(err) => {
            println!("Failed to insert into database");
            return Err(err);
        }
    }

    eprintln!("{} {} {:?} {:?}", username, user_unique_id, reg_state, sk);

    Ok(HttpResponse::Ok().finish())
}

pub fn init(config: &mut web::ServiceConfig) -> () {
    config.service(register_start).service(register_finish);

    ()
}
