use serde_json::Value;

pub fn format_reddit_message(kind: &str, data: &Value) -> String {
    match kind {
        "t4" => {
            let author = data["author"].as_str().unwrap_or("unknown");
            let subject = data["subject"].as_str().unwrap_or("(no subject)");
            let body = data["body"].as_str().unwrap_or("");
            format!(
                "\u{1F4EC} Reddit DM from **{author}**\n*{subject}*\n\n{body}"
            )
        }
        "t1" => {
            let author = data["author"].as_str().unwrap_or("unknown");
            let subreddit = data["subreddit"].as_str().unwrap_or("unknown");
            let body = data["body"].as_str().unwrap_or("");
            let context_default = format!("/r/{}", subreddit);
            let context = data["context"].as_str().unwrap_or(&context_default);
            format!(
                "\u{1F4AC} Reddit reply from **{author}** in r/{subreddit}\n\n{body}\n\n[View](https://reddit.com{context})"
            )
        }
        _ => {
            let author = data["author"].as_str().unwrap_or("unknown");
            let body = data["body"].as_str().unwrap_or(data["title"].as_str().unwrap_or(""));
            format!(
                "\u{1F514} Reddit notification from **{author}**\n\n{body}"
            )
        }
    }
}

pub fn format_tweet(
    text: &str,
    username: &str,
    display_name: &str,
    tweet_id: &str,
) -> String {
    let url = format!("https://x.com/{username}/status/{tweet_id}");
    format!(
        "\u{1F426} Mention from **@{username}** ({display_name})\n\n{text}\n\n[View]({url})"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── format_reddit_message ─────────────────────────────────────────────

    #[test]
    fn format_reddit_message_dm() {
        let data = json!({
            "author": "alice",
            "subject": "Hello there",
            "body": "This is a DM body",
        });
        let result = format_reddit_message("t4", &data);
        assert!(result.contains("\u{1F4EC}"));
        assert!(result.contains("alice"));
        assert!(result.contains("Hello there"));
        assert!(result.contains("This is a DM body"));
    }

    #[test]
    fn format_reddit_message_reply() {
        let data = json!({
            "author": "bob",
            "subreddit": "rust",
            "body": "Nice comment!",
            "context": "/r/rust/comments/abc/",
        });
        let result = format_reddit_message("t1", &data);
        assert!(result.contains("\u{1F4AC}"));
        assert!(result.contains("bob"));
        assert!(result.contains("r/rust"));
        assert!(result.contains("Nice comment!"));
        assert!(result.contains("https://reddit.com/r/rust/comments/abc/"));
    }

    #[test]
    fn format_reddit_message_fallback() {
        let data = json!({
            "author": "carol",
            "title": "Subreddit news",
            "body": "Some notification body",
        });
        let result = format_reddit_message("t2", &data);
        assert!(result.contains("\u{1F514}"));
        assert!(result.contains("carol"));
        assert!(result.contains("Some notification body"));
    }

    #[test]
    fn format_reddit_message_no_author() {
        let data = json!({});
        let result = format_reddit_message("t4", &data);
        assert!(result.contains("unknown"));
    }

    #[test]
    fn format_reddit_message_missing_fields() {
        let data = json!({});
        let result = format_reddit_message("t1", &data);
        assert!(result.contains("unknown"));
        assert!(result.contains("https://reddit.com/r/unknown"));
        assert!(!result.contains("{subreddit}"));
    }

    // ── format_tweet ──────────────────────────────────────────────────────

    #[test]
    fn format_tweet_basic() {
        let result = format_tweet(
            "Check this out!",
            "testuser",
            "Test User",
            "123456789",
        );
        assert!(result.contains("\u{1F426}"));
        assert!(result.contains("@testuser"));
        assert!(result.contains("Test User"));
        assert!(result.contains("Check this out!"));
        assert!(result.contains("https://x.com/testuser/status/123456789"));
    }

    #[test]
    fn format_tweet_special_chars() {
        let result = format_tweet(
            "Line1\nLine2\n❤️🔥",
            "u",
            "U",
            "1",
        );
        assert!(result.contains("Line1\nLine2"));
        assert!(result.contains("❤️🔥"));
    }

    #[test]
    fn format_tweet_link_format() {
        let result = format_tweet("text", "user1", "User One", "42");
        assert!(result.contains("https://x.com/user1/status/42"));
    }
}
