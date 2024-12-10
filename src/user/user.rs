use std::error::Error;

use crate::config::config::{AppConfig, AppError, AppResult};
use mongodb::{
    bson::{doc, Document},
    Client, Collection, Database,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::CredentialID;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    pub username: String,
    pub email: String,
    pub credentials: Vec<StoredCredential>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub credential_id: Vec<u8>,
    pub credential_data: Vec<u8>,
}

pub struct UserRepository {
    collection: Collection<User>,
}

impl UserRepository {
    pub async fn new(config: &AppConfig) -> AppResult<Self> {
        let client = Client::with_uri_str(&config.mongodb_uri).await?;
        let db = client.database(&config.database_name);
        let collection = db.collection("users");

        // Create unique index on username
        let index_model = mongodb::IndexModel::builder()
            .keys(doc! { "username": 1 })
            .build();
        collection.create_index(index_model, None).await?;

        Ok(Self { collection })
    }

    pub async fn create_user(&self, username: &str, email: &str) -> AppResult<User> {
        // Generate a unique user ID
        let user_id = bson::oid::ObjectId::new();

        let user = User {
            id: user_id,
            username: username.to_string(),
            email: email.to_string(),
            credentials: Vec::new(),
        };

        self.collection.insert_one(user.clone(), None).await?;

        Ok(user)
    }

    pub async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let filter = doc! { "username": username };
        let result = self.collection.find_one(filter, None).await?;
        Ok(result)
    }

    pub async fn add_credential(
        &self,
        username: &str,
        credential: StoredCredential,
    ) -> Result<(), Box<dyn Error>> {
        let filter = doc! { "username": username };
        let update = doc! {
            "$push": { "credentials": bson::to_bson(&credential)? }
        };

        self.collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
