use mongodb::error::Error as MongoError;
use std::fmt;

#[derive(Debug)]
pub enum PollError {
    MongoError(MongoError),
    PollNotFound(String),
    PollAlreadyExists(String),
    PollCreationError(String),
    PollVoteError(String),
    PollUpdateError(String),
    GeneralError(String),
    PollDeletionError(String),
    AlreadyVotedError(String),
    PollUnauthorizedAccess(String),
}

impl fmt::Display for PollError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PollError::MongoError(err) => write!(f, "MongoDB Error: {}", err),
            PollError::PollNotFound(poll_id) => write!(f, "Poll with ID '{}' not found", poll_id),
            PollError::PollAlreadyExists(title) => {
                write!(f, "Poll with title '{}' already exists", title)
            }
            PollError::PollCreationError(msg) => write!(f, "Poll creation error: {}", msg),
            PollError::PollVoteError(msg) => write!(f, "Poll vote error: {}", msg),
            PollError::PollUpdateError(msg) => write!(f, "Poll update error: {}", msg),
            PollError::GeneralError(msg) => write!(f, "Error: {}", msg),
            PollError::PollDeletionError(msg) => write!(f, "Poll deletion error: {}", msg),
            PollError::AlreadyVotedError(msg) => write!(f, "Conflict : {}", msg),
            PollError::PollUnauthorizedAccess(msg) => write!(f, "Unauthorized Access : {}", msg),
        }
    }
}

impl From<MongoError> for PollError {
    fn from(err: MongoError) -> Self {
        PollError::MongoError(err)
    }
}
