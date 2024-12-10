use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::Passkey;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub public_key: Passkey,
}
impl User {
    pub fn init(username: &String, sk: &Passkey) -> Self {
        User {
            username: username.clone(),
            public_key: sk.clone(),
        }
    }
}
