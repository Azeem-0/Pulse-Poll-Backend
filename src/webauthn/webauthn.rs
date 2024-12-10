use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use webauthn_rs::{prelude::*, Webauthn, WebauthnBuilder};

use crate::config::config::{AppConfig, AppError, AppResult};
use crate::user::user::{StoredCredential, UserRepository};

pub struct WebAuthnHandler {
    webauthn: Webauthn,
    user_repo: UserRepository,
}

#[derive(Deserialize)]
pub struct RegistrationStart {
    username: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegistrationChallenge {
    challenge: String,
    username: String,
    email: String,
}

impl WebAuthnHandler {
    pub fn new(config: &AppConfig, user_repo: UserRepository) -> AppResult<Self> {
        let webauthn = WebauthnBuilder::new(&config.webauthn_origin, &config.webauthn_rp_name)
            .map_err(|e| AppError::WebAuthnError(e.to_string()))?
            .build()
            .map_err(|e| AppError::WebAuthnError(e.to_string()))?;

        Ok(Self {
            webauthn,
            user_repo,
        })
    }

    pub async fn registration_start(
        &self,
        data: web::Json<RegistrationStart>,
    ) -> AppResult<impl Responder> {
        // Check if username already exists
        if self
            .user_repo
            .find_by_username(&data.username)
            .await?
            .is_some()
        {
            return Err(AppError::ValidationError(
                "Username already exists".to_string(),
            ));
        }

        // Create a new user
        let user = self
            .user_repo
            .create_user(&data.username, &data.email)
            .await?;

        // Generate registration options
        let (ccr, session_data) = self
            .webauthn
            .start_passkey_registration(user.id.to_hex(), &data.username, &data.email)
            .map_err(|e| AppError::WebAuthnError(e.to_string()))?;

        // Respond with challenge
        Ok(HttpResponse::Ok().json(RegistrationChallenge {
            challenge: base64_encode(&ccr.public_key_credential_rp_id),
            username: data.username.clone(),
            email: data.email.clone(),
        }))
    }

    pub async fn registration_finish(
        &self,
        username: String,
        credential: PublicKeyCredential,
    ) -> AppResult<impl Responder> {
        // Verify the credential
        let credential_registration = self
            .webauthn
            .finish_passkey_registration(&credential)
            .map_err(|e| AppError::WebAuthnError(e.to_string()))?;

        // Store the credential
        let stored_credential = StoredCredential {
            credential_id: credential_registration.credential_id.as_bytes().to_vec(),
            credential_data: credential_registration.credential.to_vec(),
        };

        // Add credential to user
        self.user_repo
            .add_credential(&username, stored_credential)
            .await?;

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Registration completed"
        })))
    }
}

// Utility function for base64 encoding
fn base64_encode(data: &[u8]) -> String {
    base64::encode(data)
}
