use std::{env, error::Error};

use dotenv::dotenv;
use mongodb::Client;

use super::user_repository::UserRepository;

pub struct MongoDB {
    pub user_repository: UserRepository,
}

impl MongoDB {
    pub async fn init() -> Result<Self, Box<dyn Error>> {
        dotenv().ok();
        let database_uri = env::var("DATABASE_URL").unwrap();

        let client = Client::with_uri_str(database_uri).await.unwrap();

        let database = client.database("passkey");

        let user_collection = database.collection("user");

        let user_reg_state_collection = database.collection("regstate");
        let user_login_state_collection = database.collection("loginstate");

        let user_repository = UserRepository::init(
            user_collection,
            user_reg_state_collection,
            user_login_state_collection,
        )
        .unwrap();

        Ok(MongoDB { user_repository })
    }
}
