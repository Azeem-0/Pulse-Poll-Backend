use mongodb::{bson::doc, results::InsertOneResult, Collection};
use webauthn_rs::prelude::Passkey;

use crate::{
    config::config::{AppResult, Error},
    models::user_model::User,
};

pub struct UserRepository {
    col: Collection<User>,
}

impl UserRepository {
    pub fn init(col: Collection<User>) -> AppResult<Self> {
        Ok(UserRepository { col })
    }

    pub async fn insert_user(&self, user: &User) -> AppResult<InsertOneResult> {
        let insert_details = self
            .col
            .insert_one(user, None)
            .await
            .expect("Failed to insert user data");
        Ok(insert_details)
    }

    pub async fn find_user(&self, username: &String) -> AppResult<Option<User>> {
        let filter = doc! { "username": username };
        let user = self
            .col
            .find_one(filter, None)
            .await
            .expect("Failed to fetch user data.");

        Ok(user)
    }

    pub async fn get_user_public_key(&self, username: &String) -> AppResult<Passkey> {
        let user = match self.find_user(username).await.unwrap() {
            Some(u) => u,
            None => {
                println!("User not found");
                return Err(Error::UserNotFound);
            }
        };

        Ok(user.public_key)
    }
}
