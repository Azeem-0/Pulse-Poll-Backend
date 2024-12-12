use std::error::Error;

use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    pub fn generate_token(username: &String) -> Result<String, Box<dyn Error>> {
        let now = Utc::now();
        let claims = Claims {
            sub: username.clone(),
            exp: (now + chrono::Duration::hours(1)).timestamp() as usize,
        };

        let secret = std::env::var("JWT_SECRET").unwrap();
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| {
            info!("Error generating token: {:?}", e);

            "no token".to_string()
        })
        .unwrap();

        Ok(token)
    }
}
