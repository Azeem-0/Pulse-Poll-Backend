use std::{env, error::Error};

use dotenv::dotenv;
use mongodb::{Client, Collection};

use crate::models::user_model::User;

pub struct MongoDB {
    pub user_collection: Collection<User>,
}

impl MongoDB {
    pub async fn init() -> Result<Self, Box<dyn Error>> {
        dotenv().ok();
        let database_uri = env::var("DATABASE_URL").unwrap();

        let client = Client::with_uri_str(database_uri).await.unwrap();

        let database = client.database("passkey");

        let user_collection = database.collection("user");

        Ok(MongoDB { user_collection })
    }
}
