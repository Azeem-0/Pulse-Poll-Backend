use crate::config::user_config::Error;

use mongodb::{
    bson::{doc, to_bson},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Collection,
};

use crate::models::user_model::{User, UserLoginState, UserRegistrationState};

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
    ) -> Result<Self, Error> {
        Ok(UserRepository {
            user_collection,
            user_login_state_collection,
            user_reg_state_collection,
        })
    }

    pub async fn insert_user(&self, user: &User) -> Result<InsertOneResult, Error> {
        self.user_collection
            .insert_one(user, None)
            .await
            .map_err(|e| Error::MongoError(e))
    }

    pub async fn find_user(&self, username: &str) -> Result<Option<User>, Error> {
        let filter = doc! { "username": username };
        self.user_collection
            .find_one(filter, None)
            .await
            .map_err(|e| Error::MongoError(e))
    }

    pub async fn get_user_credentials(&self, username: &str) -> Result<User, Error> {
        let user = match self.find_user(username).await? {
            Some(u) => u,
            None => return Err(Error::UserNotFound(username.to_string())),
        };

        Ok(user)
    }

    pub async fn store_login_state(
        &self,
        login_state: UserLoginState,
    ) -> Result<UpdateResult, Error> {
        let filter = doc! {"username": &login_state.username};

        let state_bson = to_bson(&login_state.state)
            .map_err(|_e| return Error::GeneralError("Failed to Deserialize".to_string()))?;

        let update = doc! {
            "$set": {
                "username": &login_state.username,
                "state": state_bson,
            }
        };

        let update_result = self
            .user_login_state_collection
            .update_one(
                filter,
                update,
                Some(
                    mongodb::options::UpdateOptions::builder()
                        .upsert(true)
                        .build(),
                ),
            )
            .await
            .map_err(|e| Error::MongoError(e))?;

        Ok(update_result)
    }

    pub async fn store_reg_state(
        &self,
        reg_state: UserRegistrationState,
    ) -> Result<UpdateResult, Error> {
        let filter = doc! {"username": &reg_state.username};

        let state_bson = to_bson(&reg_state.state)
            .map_err(|_e| return Error::GeneralError("Failed to Deserialize".to_string()))?;

        let update = doc! {
            "$set": {
                "username": &reg_state.username,
                "userId": &reg_state.user_id,
                "state": state_bson,
            }
        };

        let update_result = self
            .user_reg_state_collection
            .update_one(
                filter,
                update,
                Some(
                    mongodb::options::UpdateOptions::builder()
                        .upsert(true)
                        .build(),
                ),
            )
            .await
            .map_err(|e| Error::MongoError(e))?;

        Ok(update_result)
    }

    pub async fn get_reg_state(
        &self,
        username: &str,
    ) -> Result<Option<UserRegistrationState>, Error> {
        let filter = doc! { "username": username };
        self.user_reg_state_collection
            .find_one(filter, None)
            .await
            .map_err(|e| Error::RegistrationStateError(e.to_string()))
    }

    pub async fn delete_reg_state(&self, username: &str) -> Result<DeleteResult, Error> {
        let filter = doc! { "username": username };
        self.user_reg_state_collection
            .delete_one(filter, None)
            .await
            .map_err(|e| Error::RegistrationStateError(e.to_string()))
    }

    pub async fn get_login_state(&self, username: &str) -> Result<Option<UserLoginState>, Error> {
        let filter = doc! { "username": username };
        self.user_login_state_collection
            .find_one(filter, None)
            .await
            .map_err(|e| Error::LoginStateError(e.to_string()))
    }

    pub async fn delete_login_state(&self, username: &str) -> Result<DeleteResult, Error> {
        let filter = doc! { "username": username };
        self.user_login_state_collection
            .delete_one(filter, None)
            .await
            .map_err(|e| Error::LoginStateError(e.to_string()))
    }
}
