use chrono::{Duration, Utc};
use serde_json::json;

use crate::models::poll_model::Poll;

pub fn calculate_poll_results(poll: &Poll) -> serde_json::Value {
    let total_votes: usize = poll.options.iter().map(|opt| opt.votes as usize).sum();

    let options_with_percentages: Vec<_> = poll
        .options
        .iter()
        .map(|opt| {
            let percentage = if total_votes > 0 {
                (opt.votes as f64 / total_votes as f64) * 100.0
            } else {
                0.0
            };
            json!({
                "option_id": opt.option_id,
                "text": opt.text,
                "votes": opt.votes,
                "percentage": percentage
            })
        })
        .collect();

    let time_elapsed = format_duration(Utc::now().signed_duration_since(poll.created_at));

    json!({
        "pollId": poll.poll_id,
        "title": poll.title,
        "totalVotes": total_votes,
        "options": options_with_percentages,
        "timeElapsed": time_elapsed,
    })
}

pub fn format_duration(duration: Duration) -> String {
    let secs = duration.num_seconds();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
}
