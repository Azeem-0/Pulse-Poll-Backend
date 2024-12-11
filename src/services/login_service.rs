use actix_web::{
    post,
    web::{self, Data, Json, Path},
    HttpResponse,
};
use log::info;
use webauthn_rs::{
    prelude::{Passkey, PublicKeyCredential, RequestChallengeResponse},
    Webauthn,
};

use crate::{
    config::config::{AppResult, Error},
    db::mongodb_repository::MongoDB,
    models::user_model::UserLoginState,
};

#[post("/start/{username}")]
pub async fn authentication_start(
    username: Path<String>,
    webauthn: Data<Webauthn>,
    db: Data<MongoDB>,
) -> AppResult<Json<RequestChallengeResponse>> {
    let user_credentials = db
        .user_repository
        .get_user_credentials(&username)
        .await
        .map_err(|_| Error::UserNotFound)
        .unwrap();

    let mut allow_credentials: Vec<Passkey> = Vec::new();
    match serde_json::from_value(user_credentials.sk) {
        Ok(val) => {
            allow_credentials.push(val);
        }
        Err(err) => {
            println!("{:?}", err);
            return Err(Error::UserNotFound);
        }
    };

    let (rcr, auth_state) = webauthn
        .start_passkey_authentication(&allow_credentials)
        .map_err(|e| {
            info!("challenge_authenticate -> {:?}", e);
            Error::Unknown(e)
        })?;

    let login_state_value = match serde_json::to_value(&auth_state) {
        Ok(value) => value,
        Err(_) => {
            return Err(Error::UserHasNoCredentials);
        }
    };

    let login_state = UserLoginState {
        username: username.clone(),
        state: login_state_value,
    };

    db.user_repository
        .store_login_state(login_state)
        .await
        .unwrap();

    Ok(Json(rcr))
}

#[post("/finish/{username}")]
pub async fn authentication_finish(
    auth: Json<PublicKeyCredential>,
    webauthn: Data<Webauthn>,
    username: Path<String>,
    db: Data<MongoDB>,
) -> AppResult<HttpResponse> {
    let user_login_state = match db.user_repository.get_login_state(&username).await.unwrap() {
        Some(reg_state) => reg_state,
        None => {
            return Err(Error::UserNotFound);
        }
    };

    let username = user_login_state.username.clone();
    let state = user_login_state.state.clone();

    let auth_state = match serde_json::from_value(state) {
        Ok(reg) => reg,
        Err(_) => {
            return Err(Error::UserNotFound);
        }
    };

    let _auth_result = webauthn
        .finish_passkey_authentication(&auth, &auth_state)
        .map_err(|e| {
            info!("challenge_register -> {:?}", e);
            Error::BadRequest(e)
        })?;

    db.user_repository
        .delete_login_state(&username)
        .await
        .unwrap();

    info!("Authentication Successful!");
    Ok(HttpResponse::Ok().finish())
}
pub fn init(config: &mut web::ServiceConfig) {
    config
        .service(authentication_start)
        .service(authentication_finish);
}
