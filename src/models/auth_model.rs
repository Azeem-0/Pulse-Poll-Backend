use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    pub username: String,
}
