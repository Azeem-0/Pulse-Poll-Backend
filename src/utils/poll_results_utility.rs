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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::poll_model::{OptionItem, Poll};
    use chrono::{Duration, Utc};

    #[test]
    fn test_format_duration() {
        let duration = Duration::seconds(90061); // 1 day, 1 hour, 1 minute, 1 second
        let result = format_duration(duration);
        assert_eq!(result, "1d 1h 1m 1s");

        let duration_zero = Duration::seconds(0);
        let result_zero = format_duration(duration_zero);
        assert_eq!(result_zero, "0d 0h 0m 0s");

        let duration_negative = Duration::seconds(-3661); // Negative duration
        let result_negative = format_duration(duration_negative);
        assert_ne!(result_negative, "-1d -22h -59m -59s"); // Edge case
    }

    #[test]
    fn test_calculate_poll_results() {
        // Setup a sample poll
        let poll = Poll {
            username: String::from("Azeem"),
            is_active: true,
            updated_at: Utc::now() - Duration::hours(2),
            voters: vec![],
            poll_id: "poll123".to_string(),
            title: "Favorite Programming Language".to_string(),
            created_at: Utc::now() - Duration::hours(2),
            options: vec![
                OptionItem {
                    option_id: "1".to_string(),
                    text: "Rust".to_string(),
                    votes: 70,
                },
                OptionItem {
                    option_id: "2".to_string(),
                    text: "Python".to_string(),
                    votes: 30,
                },
            ],
        };

        let result = calculate_poll_results(&poll);

        // Check total votes
        assert_eq!(result["totalVotes"], 100);

        // Check individual options
        let options = result["options"].as_array().unwrap();
        assert_eq!(options.len(), 2);

        assert_eq!(options[0]["option_id"], "1");
        assert_eq!(options[0]["text"], "Rust");
        assert_eq!(options[0]["votes"], 70);
        assert_eq!(options[0]["percentage"], 70.0);

        assert_eq!(options[1]["option_id"], "2");
        assert_eq!(options[1]["text"], "Python");
        assert_eq!(options[1]["votes"], 30);
        assert_eq!(options[1]["percentage"], 30.0);

        // Check time elapsed format
        let time_elapsed = result["timeElapsed"].as_str().unwrap();
        assert!(time_elapsed.contains("2h")); // Ensure it contains ~2 hours
    }

    #[test]
    fn test_calculate_poll_results_no_votes() {
        // Setup a poll with no votes
        let poll = Poll {
            username: String::from("Azeem"),
            is_active: true,
            updated_at: Utc::now() - Duration::hours(2),
            voters: vec![],
            poll_id: "poll123".to_string(),
            title: "Favorite Programming Language".to_string(),
            created_at: Utc::now() - Duration::hours(2),
            options: vec![
                OptionItem {
                    option_id: "1".to_string(),
                    text: "Rust".to_string(),
                    votes: 70,
                },
                OptionItem {
                    option_id: "2".to_string(),
                    text: "Python".to_string(),
                    votes: 30,
                },
            ],
        };

        let result = calculate_poll_results(&poll);

        assert_eq!(result["totalVotes"], 100);

        let options = result["options"].as_array().unwrap();
        assert_eq!(options[0]["percentage"], 70.0);
        assert_eq!(options[1]["percentage"], 30.0);
    }
}
