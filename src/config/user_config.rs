use mongodb::error::Error as MongoError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    MongoError(MongoError),
    UserNotFound(String),
    UserAlreadyExists(String),
    RegistrationStateError(String),
    LoginStateError(String),
    GeneralError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MongoError(err) => write!(f, "MongoDB Error: {}", err),
            Error::UserNotFound(username) => write!(f, "User '{}' not found", username),
            Error::UserAlreadyExists(username) => write!(f, "User '{}' already exists", username),
            Error::RegistrationStateError(msg) => write!(f, "Registration state error: {}", msg),
            Error::LoginStateError(msg) => write!(f, "Login state error: {}", msg),
            Error::GeneralError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl From<MongoError> for Error {
    fn from(err: MongoError) -> Self {
        Error::MongoError(err)
    }
}
