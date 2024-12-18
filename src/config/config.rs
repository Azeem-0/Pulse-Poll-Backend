use std::env;

use dotenv::dotenv;

use serde::Deserialize;

// Application-wide configuration
#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub mongodb_uri: String,
    pub database_name: String,
    pub jwt_secret: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        dotenv().ok();

        Self {
            mongodb_uri: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
            database_name: env::var("DATABASE_NAME")
                .unwrap_or_else(|_| "polling_application".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "thisisthemostsecuresecret".to_string()),
        }
    }
}
