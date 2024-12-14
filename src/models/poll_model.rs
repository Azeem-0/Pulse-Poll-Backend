use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Poll {
    pub poll_id: String,
    pub username: String,
    pub title: String,
    pub options: Vec<OptionItem>,
    pub is_active: bool,
    pub voters: Vec<VoteHistory>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionItem {
    pub option_id: String,
    pub text: String,
    pub votes: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteHistory {
    pub username: String,
    pub option_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollResults {
    poll_id: String,
    title: String,
    options: Vec<ResultsOptionItem>,
    total_votes: u32,
    time_elapsed: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultsOptionItem {
    option_id: String,
    text: String,
    votes: u32,
    percentage: u32,
}

#[derive(Deserialize, Debug)]
pub struct PollQueryParams {
    pub live: Option<bool>,
    pub closed: Option<bool>,
    pub creator: Option<String>,
}
