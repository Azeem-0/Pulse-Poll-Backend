use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix_session::{SessionGetError, SessionInsertError};
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use webauthn_rs::prelude::{PasskeyAuthentication, PasskeyRegistration, WebauthnError};

// Application-wide configuration
#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub mongodb_uri: String,
    pub database_name: String,
    pub webauthn_origin: String,
    pub webauthn_rp_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mongodb_uri:
                "mongodb+srv://azeemshaik025:Azeem%404659@cluster0.84u62sl.mongodb.net/passkey"
                    .to_string(),
            database_name: "passkey_auth".to_string(),
            webauthn_origin: "http://localhost:8080".to_string(),
            webauthn_rp_name: "Passkey Authentication Demo".to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown webauthn error")]
    Unknown(WebauthnError),
    #[error("Corrupt session")]
    SessionGet(#[from] SessionGetError),
    #[error("Corrupt session")]
    SessionInsert(#[from] SessionInsertError),
    #[error("Corrupt session")]
    CorruptSession,
    #[error("Bad request")]
    BadRequest(#[from] WebauthnError),
    #[error("User not found")]
    UserNotFound,
    #[error("User already registered")]
    UserAlreadyRegistered,
    #[error("User has no credentials")]
    UserHasNoCredentials,
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub type RegistrationStateStore = Arc<Mutex<HashMap<String, (String, Uuid, PasskeyRegistration)>>>;
pub type LoginStateStore = Arc<Mutex<HashMap<String, (String, Uuid, PasskeyAuthentication)>>>;
