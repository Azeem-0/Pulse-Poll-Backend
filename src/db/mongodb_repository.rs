use std::error::Error;

use dotenv::dotenv;
use mongodb::Client;

use super::{poll_repository::PollRepository, user_repository::UserRepository};

pub struct MongoDB {
    pub user_repository: UserRepository,
    pub poll_repository: PollRepository,
}

impl MongoDB {
    pub async fn init(mongo_uri: &str, database_name: &str) -> Result<Self, Box<dyn Error>> {
        dotenv().ok();

        let client = Client::with_uri_str(mongo_uri)
            .await
            .map_err(|e| Box::new(e))?;

        let database = client.database(database_name);

        let user_collection = database.collection("user");
        let poll_collection = database.collection("poll");
        let user_reg_state_collection = database.collection("regstate");
        let user_login_state_collection = database.collection("loginstate");

        let user_repository = UserRepository::init(
            user_collection,
            user_reg_state_collection,
            user_login_state_collection,
        )
        .unwrap();

        let poll_repository = PollRepository::init(poll_collection).unwrap();

        Ok(MongoDB {
            user_repository,
            poll_repository,
        })
    }
}
