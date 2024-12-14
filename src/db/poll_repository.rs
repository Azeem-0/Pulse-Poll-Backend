use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::options::UpdateOptions;
use mongodb::results::{DeleteResult, InsertOneResult, UpdateResult};
use mongodb::Collection;

use crate::config::poll_config::PollError;
use crate::models::poll_model::Poll;

pub struct PollRepository {
    poll_collection: Collection<Poll>,
}
impl PollRepository {
    pub fn init(poll_collection: Collection<Poll>) -> Result<Self, PollError> {
        Ok(PollRepository { poll_collection })
    }

    pub async fn create_poll(&self, poll: &Poll) -> Result<InsertOneResult, PollError> {
        self.poll_collection
            .insert_one(poll, None)
            .await
            .map_err(|e| PollError::PollCreationError(e.to_string()))
    }

    pub async fn get_poll_by_id(&self, poll_id: &str) -> Result<Option<Poll>, PollError> {
        let filter = doc! {"pollId" : poll_id};

        self.poll_collection
            .find_one(filter, None)
            .await
            .map_err(|e| PollError::MongoError(e))
    }

    pub async fn get_all_polls(&self) -> Result<Vec<Poll>, mongodb::error::Error> {
        let cursor = self
            .poll_collection
            .find(None, None)
            .await
            .map_err(|e| PollError::from(e))
            .unwrap();

        let polls: Result<Vec<Poll>, mongodb::error::Error> = cursor.try_collect().await;

        polls
    }

    pub async fn check_user_vote_in_poll(
        &self,
        username: &str,
        poll_id: &str,
    ) -> Result<bool, PollError> {
        let filter = doc! { "pollId": poll_id };

        let poll = self
            .poll_collection
            .find_one(filter, None)
            .await
            .map_err(|err| PollError::GeneralError(format!("Failed to find poll: {}", err)))?;

        match poll {
            Some(poll) => {
                let user_voted = poll
                    .voters
                    .iter()
                    .any(|vote_history| vote_history.username == username);

                if user_voted {
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
            None => {
                return Err(PollError::GeneralError("Poll not found".to_string()));
            }
        };
    }

    pub async fn cast_vote_to_poll_by_id(
        &self,
        poll_id: &str,
        option_id: &str,
        username: &str,
    ) -> Result<UpdateResult, PollError> {
        let update_poll = doc! {
            "$inc": {
                "options.$[option].votes": 1
            },
            "$addToSet": {
                "voters": {
                    "username":username,
                    "optionId":option_id
                }
            }
        };

        let array_filters = vec![doc! { "option.optionId": option_id }];

        let filter = doc! { "pollId": poll_id };

        let update_result = self
            .poll_collection
            .update_one(
                filter,
                update_poll,
                Some(
                    UpdateOptions::builder()
                        .array_filters(Some(array_filters))
                        .build(),
                ),
            )
            .await
            .map_err(|err| PollError::GeneralError(format!("Failed to cast vote: {}", err)))?;

        Ok(update_result)
    }

    pub async fn change_vote_in_poll_by_id(
        &self,
        poll_id: &str,
        new_option_id: &str,
        username: &str,
    ) -> Result<UpdateResult, PollError> {
        let poll = self.get_poll_by_id(poll_id).await.unwrap();

        if let Some(poll) = poll {
            let previous_vote = poll
                .voters
                .iter()
                .find(|vote_history| vote_history.username == username);

            if let Some(vote_history) = previous_vote {
                let previous_option_id = &vote_history.option_id;

                if previous_option_id == new_option_id {
                    return Err(PollError::AlreadyVotedError(
                        "Already voted to the option in the poll.".to_string(),
                    ));
                }

                let update_poll = doc! {
                    "$inc": {
                        "options.$[prevOption].votes": -1,
                        "options.$[newOption].votes": 1
                    },
                    "$set": {

                        "voters.$[elem].optionId": new_option_id
                    }
                };

                let array_filters = vec![
                    doc! { "prevOption.optionId": previous_option_id },
                    doc! { "newOption.optionId": new_option_id },
                    doc! { "elem.username": username },
                ];

                let filter = doc! { "pollId": poll_id };

                let update_result = self
                    .poll_collection
                    .update_one(
                        filter,
                        update_poll,
                        Some(
                            UpdateOptions::builder()
                                .array_filters(Some(array_filters))
                                .build(),
                        ),
                    )
                    .await
                    .map_err(|err| {
                        PollError::GeneralError(format!("Failed to change vote: {}", err))
                    })?;

                return Ok(update_result);
            }
        }

        Err(PollError::GeneralError(
            "Poll not found or user did not vote".to_string(),
        ))
    }

    pub async fn remove_poll_by_id(&self, poll_id: &str) -> Result<DeleteResult, PollError> {
        let query = doc! {"pollId":poll_id};
        self.poll_collection
            .delete_one(query, None)
            .await
            .map_err(|e| PollError::PollDeletionError(e.to_string()))
    }

    pub async fn close_poll_by_id(
        &self,
        poll_id: &str,
        username: &str,
    ) -> Result<UpdateResult, PollError> {
        if self
            .check_user_ownership_on_poll(poll_id, username)
            .await
            .unwrap()
        {
            let query = doc! { "pollId": poll_id };
            let update = doc! {
                "$set": {
                    "isActive": false
                }
            };
            self.poll_collection
                .update_one(query, update, None)
                .await
                .map_err(|e| PollError::PollUpdateError(e.to_string()))
        } else {
            return Err(PollError::PollUnauthorizedAccess(
                "Poll can be deleted only by the creator.".to_string(),
            ));
        }
    }

    pub async fn reset_poll_by_id(
        &self,
        poll_id: &str,
        username: &str,
    ) -> Result<UpdateResult, PollError> {
        if self
            .check_user_ownership_on_poll(poll_id, username)
            .await
            .unwrap()
        {
            let filter = doc! {"pollId":poll_id};

            let update = doc! {
                "$set":{
                    "options.$[].votes": 0,
                    "voters":[]
                }
            };

            let update_result = self
                .poll_collection
                .update_one(filter, update, None)
                .await
                .map_err(|err| {
                    PollError::GeneralError(format!("Failed to reset votes: {}", err))
                })?;

            Ok(update_result)
        } else {
            return Err(PollError::PollUnauthorizedAccess(
                "Only the creator can reset the votes.".to_string(),
            ));
        }
    }

    async fn check_user_ownership_on_poll(
        &self,
        poll_id: &str,
        username: &str,
    ) -> Result<bool, PollError> {
        let poll = match self.get_poll_by_id(poll_id).await.unwrap() {
            Some(poll) => poll,
            None => {
                return Err(PollError::PollNotFound("Poll not found".to_string()));
            }
        };

        if poll.username == username {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
