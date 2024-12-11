use actix_web::{
    post,
    web::{self, Data, Json, Path},
    HttpResponse,
};

use crate::models::user_model::{User, UserRegistrationState};
use crate::{
    config::config::{AppResult, Error},
    db::mongodb_repository::MongoDB,
};
use log::info;
use uuid::Uuid;
use webauthn_rs::prelude::{CreationChallengeResponse, RegisterPublicKeyCredential, Webauthn};

#[post("/start/{username}")]
pub async fn register_start(
    username: Path<String>,
    webauthn: Data<Webauthn>,
    db: Data<MongoDB>,
) -> AppResult<Json<CreationChallengeResponse>> {
    let user = db.user_repository.find_user(&username).await.unwrap();

    if let Some(_u) = user {
        return Err(Error::UserAlreadyRegistered);
    }

    let user_unique_id = Uuid::new_v4();

    let (ccr, reg_state) = webauthn
        .start_passkey_registration(user_unique_id, &username, &username, None)
        .map_err(|e| {
            info!("challenge_register -> {:?}", e);
            Error::Unknown(e)
        })?;

    let reg_state_value = match serde_json::to_value(&reg_state) {
        Ok(value) => value,
        Err(_) => {
            return Err(Error::UserHasNoCredentials);
        }
    };

    let user_reg_state = UserRegistrationState {
        user_id: user_unique_id.to_string(),
        username: username.clone(),
        state: reg_state_value,
    };

    db.user_repository
        .store_reg_state(user_reg_state)
        .await
        .unwrap();

    Ok(Json(ccr))
}

#[post("/finish/{username}")]
pub async fn register_finish(
    req: web::Json<RegisterPublicKeyCredential>,
    webauthn: Data<Webauthn>,
    username: Path<String>,
    db: Data<MongoDB>,
) -> AppResult<HttpResponse> {
    let user_reg_state = match db.user_repository.get_reg_state(&username).await.unwrap() {
        Some(reg_state) => reg_state,
        None => {
            return Err(Error::UserNotFound);
        }
    };
    let state = user_reg_state.state.clone();

    let reg_state = match serde_json::from_value(state) {
        Ok(reg) => reg,
        Err(_) => {
            return Err(Error::UserNotFound);
        }
    };

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

    db.user_repository
        .delete_reg_state(&username)
        .await
        .unwrap();

    Ok(HttpResponse::Ok().finish())
}

pub fn init(config: &mut web::ServiceConfig) -> () {
    config.service(register_start).service(register_finish);

    ()
}
