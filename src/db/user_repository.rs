use mongodb::{
    bson::doc,
    results::{DeleteResult, InsertOneResult},
    Collection,
};

use crate::{
    config::config::{AppResult, Error},
    models::user_model::{User, UserLoginState, UserRegistrationState},
};

pub struct UserRepository {
    pub user_collection: Collection<User>,
    pub user_reg_state_collection: Collection<UserRegistrationState>,
    pub user_login_state_collection: Collection<UserLoginState>,
}

impl UserRepository {
    pub fn init(
        user_collection: Collection<User>,
        user_reg_state_collection: Collection<UserRegistrationState>,
        user_login_state_collection: Collection<UserLoginState>,
    ) -> AppResult<Self> {
        Ok(UserRepository {
            user_collection,
            user_login_state_collection,
            user_reg_state_collection,
        })
    }

    pub async fn insert_user(&self, user: &User) -> AppResult<InsertOneResult> {
        let insert_details = self
            .user_collection
            .insert_one(user, None)
            .await
            .expect("Failed to insert user data");
        Ok(insert_details)
    }

    pub async fn find_user(&self, username: &String) -> AppResult<Option<User>> {
        let filter = doc! { "username": username };
        let user = self
            .user_collection
            .find_one(filter, None)
            .await
            .expect("Failed to fetch user data.");

        Ok(user)
    }

    pub async fn get_user_credentials(&self, username: &String) -> AppResult<User> {
        let user = match self.find_user(username).await.unwrap() {
            Some(u) => u,
            None => {
                println!("User not found");
                return Err(Error::UserNotFound);
            }
        };

        Ok(user)
    }

    // state management database logic

    pub async fn store_reg_state(
        &self,
        reg_state: UserRegistrationState,
    ) -> AppResult<InsertOneResult> {
        let insert_details = self
            .user_reg_state_collection
            .insert_one(reg_state, None)
            .await
            .expect("Error storing the registration state");

        Ok(insert_details)
    }

    pub async fn get_reg_state(
        &self,
        username: &String,
    ) -> AppResult<Option<UserRegistrationState>> {
        let filter = doc! { "username": username };
        let reg_state = self
            .user_reg_state_collection
            .find_one(filter, None)
            .await
            .expect("Faied to retrive user registration state");
        Ok(reg_state)
    }

    pub async fn delete_reg_state(&self, username: &String) -> AppResult<DeleteResult> {
        let filter = doc! {"username" : username};
        let delete_details = self
            .user_reg_state_collection
            .delete_one(filter, None)
            .await
            .expect("Failed to delete user registration state");
        Ok(delete_details)
    }

    pub async fn store_login_state(
        &self,
        login_state: UserLoginState,
    ) -> AppResult<InsertOneResult> {
        let insert_details = self
            .user_login_state_collection
            .insert_one(login_state, None)
            .await
            .expect("Failed to insert login state");

        Ok(insert_details)
    }

    pub async fn get_login_state(&self, username: &String) -> AppResult<Option<UserLoginState>> {
        let filter = doc! {"username" : username};

        let login_state = self
            .user_login_state_collection
            .find_one(filter, None)
            .await
            .expect("failed to retrieve user login state");

        Ok(login_state)
    }

    pub async fn delete_login_state(&self, username: &String) -> AppResult<DeleteResult> {
        let filter = doc! {"username":username};

        let delete_details = self
            .user_login_state_collection
            .delete_one(filter, None)
            .await
            .expect("Failed to delete user login state");

        Ok(delete_details)
    }
}
