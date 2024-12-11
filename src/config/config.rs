use std::env;

use serde::Deserialize;

// Application-wide configuration
#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub mongodb_uri: String,
    pub database_name: String,
    pub rp_id: String,
    pub rp_name: String,
    pub rp_origin_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mongodb_uri: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
            database_name: env::var("DATABASE_NAME").unwrap_or_else(|_| "passkey_auth".to_string()),
            rp_origin_url: env::var("RP_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            rp_name: env::var("RP_NAME")
                .unwrap_or_else(|_| "Passkey Authentication Demo".to_string()),
            rp_id: env::var("RP_ID").unwrap_or_else(|_| "localhost".to_string()),
        }
    }
}
