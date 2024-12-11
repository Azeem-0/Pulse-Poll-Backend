use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use webauthn_rs::prelude::Passkey;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub username: String,
    pub sk: serde_json::Value,
}
impl User {
    pub fn init(username: &String, sk: &Passkey) -> Self {
        User {
            username: username.clone(),
            sk: serde_json::to_value(sk.clone()).unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserRegistrationState {
    pub username: String,
    pub user_id: String,
    pub state: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserLoginState {
    pub user_id: String,
    pub state: serde_json::Value,
}
