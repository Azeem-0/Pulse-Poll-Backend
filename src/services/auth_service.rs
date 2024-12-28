use crate::models::user_model::{User, UserRegistrationState};
use crate::utils::jwt_token_generation::Claims;
use crate::{db::mongodb_repository::MongoDB, models::user_model::UserLoginState};
use actix_web::cookie::time::Duration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::Responder;
use uuid::Uuid;
use webauthn_rs::prelude::{Passkey, PublicKeyCredential, RegisterPublicKeyCredential, Webauthn};

use actix_web::{
    post,
    web::{self, Data, Json, Path},
    HttpResponse,
};
use log::info;

#[post("/register/start/{username}")]
async fn register_start(
    username: Path<String>,
    webauthn: Data<Webauthn>,
    db: Data<MongoDB>,
) -> impl Responder {
    match db.user_repository.find_user(&username).await {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().body("User is already registered.");
        }
        Ok(None) => {}
        Err(_) => {
            return HttpResponse::InternalServerError().body("Error checking user registration.");
        }
    }

    let user_unique_id = Uuid::new_v4();

    let (ccr, reg_state) =
        match webauthn.start_passkey_registration(user_unique_id, &username, &username, None) {
            Ok(result) => result,
            Err(e) => {
                info!("Error starting passkey registration -> {:?}", e);
                return HttpResponse::InternalServerError()
                    .body("Failed to start registration process.");
            }
        };

    let reg_state_value = match serde_json::to_value(&reg_state) {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to serialize registration state.");
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

    HttpResponse::Ok().json(ccr)
}

#[post("/register/finish/{username}")]
async fn register_finish(
    req: web::Json<RegisterPublicKeyCredential>,
    webauthn: Data<Webauthn>,
    username: Path<String>,
    db: Data<MongoDB>,
) -> impl Responder {
    let user_reg_state = match db.user_repository.get_reg_state(&username).await {
        Ok(Some(reg_state)) => reg_state,
        Ok(None) => {
            return HttpResponse::Unauthorized().body("Registration state not found for the user.");
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Error while retrieving registration state.");
        }
    };

    let reg_state = match serde_json::from_value(user_reg_state.state.clone()) {
        Ok(reg) => reg,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to deserialize the registration state.");
        }
    };

    let sk = match webauthn.finish_passkey_registration(&req, &reg_state) {
        Ok(passkey) => passkey,
        Err(e) => {
            println!("Error during registration finish -> {:?}", e);
            if let Err(_) = db.user_repository.delete_reg_state(&username).await {
                return HttpResponse::InternalServerError()
                    .body("Error registering user, and failed to clean up registration state.");
            }
            return HttpResponse::BadRequest()
                .body("Failed to finish the passkey registration process.");
        }
    };

    let user = User::init(&username, &sk);

    if let Err(_) = db.user_repository.insert_user(&user).await {
        return HttpResponse::InternalServerError()
            .body("Failed to insert user data into the database. Please try registering again.");
    }

    if let Err(_) = db.user_repository.delete_reg_state(&username).await {
        return HttpResponse::InternalServerError()
            .body("User registered successfully, but failed to clean up registration state.");
    }

    HttpResponse::Ok().body("User registered successfully.")
}

#[post("/login/start/{username}")]
async fn authentication_start(
    username: Path<String>,
    webauthn: Data<Webauthn>,
    db: Data<MongoDB>,
) -> impl Responder {
    let user_credentials = match db.user_repository.get_user_credentials(&username).await {
        Ok(credentials) => credentials,
        Err(_) => {
            info!("User not found during authentication start: {}", username);
            return HttpResponse::NotFound().body("User not found.");
        }
    };

    let mut allow_credentials: Vec<Passkey> = Vec::new();
    match serde_json::from_value(user_credentials.sk) {
        Ok(val) => {
            allow_credentials.push(val);
        }
        Err(err) => {
            info!(
                "Failed to deserialize user credentials for {}: {:?}",
                username, err
            );
            return HttpResponse::InternalServerError()
                .body("Failed to deserialize user credentials.");
        }
    };

    let (rcr, auth_state) = match webauthn.start_passkey_authentication(&allow_credentials) {
        Ok(result) => result,
        Err(e) => {
            info!("Failed to start authentication for {}: {:?}", username, e);
            return HttpResponse::InternalServerError().body("Authentication challenge failed.");
        }
    };

    let login_state_value = match serde_json::to_value(&auth_state) {
        Ok(value) => value,
        Err(_) => {
            info!("Failed to serialize authentication state for {}", username);
            return HttpResponse::InternalServerError()
                .body("Failed to serialize authentication state.");
        }
    };

    let login_state = UserLoginState {
        username: username.clone(),
        state: login_state_value,
    };

    if let Err(err) = db.user_repository.store_login_state(login_state).await {
        info!("Failed to store login state for {}: {:?}", username, err);
        return HttpResponse::InternalServerError().body("Failed to store login state.");
    }

    HttpResponse::Ok().json(rcr)
}

#[post("/login/finish/{username}")]
async fn authentication_finish(
    auth: Json<PublicKeyCredential>,
    webauthn: Data<Webauthn>,
    username: Path<String>,
    db: Data<MongoDB>,
) -> impl Responder {
    let user_login_state = match db.user_repository.get_login_state(&username).await {
        Ok(Some(reg_state)) => reg_state,
        Ok(None) => {
            info!("No login state found for user: {}", username);
            return HttpResponse::Unauthorized().body("User doesn't have an active login state.");
        }
        Err(err) => {
            info!(
                "Error retrieving login state for user {}: {:?}",
                username, err
            );
            return HttpResponse::InternalServerError().body("Failed to retrieve login state.");
        }
    };

    let state = user_login_state.state.clone();

    let auth_state = match serde_json::from_value(state) {
        Ok(reg) => reg,
        Err(_) => {
            info!(
                "Failed to deserialize authentication state for user: {}",
                username
            );
            return HttpResponse::InternalServerError()
                .body("Failed to deserialize authentication state.");
        }
    };

    let _auth_result = match webauthn.finish_passkey_authentication(&auth, &auth_state) {
        Ok(result) => result,
        Err(err) => {
            println!(
                "Authentication challenge failed for user {}: {:?}",
                username, err
            );
            if let Err(err) = db.user_repository.delete_login_state(&username).await {
                info!(
                    "Authentication Failed , and error deleting login state for user {}: {:?}",
                    username, err
                );
                return HttpResponse::InternalServerError().body("Failed to clean up login state.");
            }
            return HttpResponse::BadRequest().body("Authentication failed.");
        }
    };

    if let Err(err) = db.user_repository.delete_login_state(&username).await {
        info!(
            "Error deleting login state for user {}: {:?}",
            username, err
        );
        return HttpResponse::InternalServerError().body("Failed to clean up login state.");
    }

    let token = Claims::generate_token(&username).unwrap();

    let cookie = Cookie::build("token", token)
        .path("/")
        .http_only(true)
        .max_age(Duration::hours(1))
        .same_site(actix_web::cookie::SameSite::None)
        .secure(true)
        .finish();

    info!("Authentication successful for user: {}", username);

    HttpResponse::Ok()
        .cookie(cookie)
        .body("Logged in successfully.")
}

#[post("/logout")]
async fn logout() -> impl Responder {
    let cookie = Cookie::build("token", "")
        .path("/")
        .http_only(true)
        .same_site(SameSite::None)
        .max_age(Duration::seconds(-1))
        .secure(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .body("Logged out successfully")
}

pub fn init(config: &mut web::ServiceConfig) -> () {
    config
        .service(register_start)
        .service(register_finish)
        .service(authentication_start)
        .service(authentication_finish)
        .service(logout);

    ()
}
