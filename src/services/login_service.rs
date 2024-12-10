use std::clone;

use actix_web::{
    post,
    web::{self, Data, Json},
    HttpResponse,
};
use log::info;
use webauthn_rs::{prelude::PublicKeyCredential, Webauthn};

use crate::{
    config::config::{AppResult, Error, LoginStateStore, RegistrationStateStore},
    db::mongodb_repository::MongoDB,
    models::auth_model::RegisterRequest,
};

#[post("/login-start")]
pub async fn authentication_start(
    data: web::Json<RegisterRequest>,
    webauthn: Data<Webauthn>,
    login_store: Data<LoginStateStore>,
    db: Data<MongoDB>,
) -> AppResult<HttpResponse> {
    let username = &data.username;
    let mut session = login_store.lock().unwrap();

    session.remove("auth_state");

    // retrieve the user's public key from the database, perform a database operation here.

    let sk = db
        .user_repository
        .get_user_public_key(username)
        .await
        .unwrap();

    // let (rcr, auth_state) = webauthn
    // .start_passkey_authentication(allow_credentials)
    // .map_err(|e| {
    //     info!("challenge_authenticate -> {:?}", e);
    //     Error::Unknown(e)
    // })?;

    // session.insert("auth_state", (username.clone(),user_unique_id, auth_state))?;

    // Ok(Json(rcr))

    Ok(HttpResponse::Ok().json("Authentication Api for the polling application."))
}

#[post("/login-finish")]
pub async fn authentication_finish(
    auth: Json<PublicKeyCredential>,
    webauthn: Data<Webauthn>,
    login_store: Data<LoginStateStore>,
) -> AppResult<HttpResponse> {
    let mut session = login_store.lock().unwrap();

    // let (username, user_unique_id, auth_state) = session.get("auth_state").unwrap();

    let (username, user_unique_id, auth_state) = match &session.get("auth_state") {
        Some((username, user_unique_id, auth_state)) => {
            (username.clone(), user_unique_id.clone(), auth_state.clone())
        }
        None => return Err(Error::CorruptSession),
    };

    session.remove("auth_state");

    let auth_result = webauthn
        .finish_passkey_authentication(&auth, &auth_state)
        .map_err(|e| {
            info!("challenge_register -> {:?}", e);
            Error::BadRequest(e)
        })?;

    // // Update the credential counter, if possible.
    // users_guard
    //     .keys
    //     .get_mut(&user_unique_id)
    //     .map(|keys| {
    //         keys.iter_mut().for_each(|sk| {
    //             // This will update the credential if it's the matching
    //             // one. Otherwise it's ignored. That is why it is safe to
    //             // iterate this over the full list.
    //             sk.update_credential(&auth_result);
    //         })
    //     })
    //     .ok_or(Error::UserHasNoCredentials)?;

    info!("Authentication Successful!");
    Ok(HttpResponse::Ok().finish())
}
pub fn init(config: &mut web::ServiceConfig) {
    config
        .service(authentication_start)
        .service(authentication_finish);
}
