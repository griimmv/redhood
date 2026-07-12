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
            let context = data["context"].as_str().unwrap_or("/r/{subreddit}");
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
