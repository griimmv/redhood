use crate::reddit::auth::RedditAuth;
use anyhow::{Context, Result};
use serde_json::Value;

pub struct RedditInbox<'a> {
    auth: &'a RedditAuth,
}

impl<'a> RedditInbox<'a> {
    pub fn new(auth: &'a RedditAuth) -> Self {
        Self { auth }
    }

    pub fn fetch_unread(&self) -> Result<Vec<RedditNotification>> {
        let token = self.auth.get_token()?;
        let client = reqwest::blocking::Client::new();
        let url = format!(
            "https://oauth.reddit.com/api/v1/message/inbox?limit=25&mark=false"
        );
        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", "linux:redhood:v0.1.0 (by /u/redhood)")
            .send()
            .context("Failed to fetch Reddit inbox")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            anyhow::bail!("Reddit inbox request failed ({}): {body}", status.as_u16());
        }

        let body: Value = resp.json()?;
        let mut notifications = Vec::new();

        if let Some(children) = body["data"]["children"].as_array() {
            for child in children {
                let kind = child["kind"].as_str().unwrap_or("");
                let data = &child["data"];
                let id = data["id"].as_str().unwrap_or("").to_string();
                let author = data["author"].as_str().unwrap_or("unknown").to_string();
                let body_text = data["body"].as_str().unwrap_or("").to_string();
                let subject = data["subject"].as_str().unwrap_or("").to_string();
                let subreddit_val = data["subreddit"].as_str().unwrap_or("").to_string();
                let context = data["context"].as_str().unwrap_or("").to_string();
                let is_new = data["new"].as_bool().unwrap_or(false);
                let created_utc = data["created_utc"].as_f64().unwrap_or(0.0);

                notifications.push(RedditNotification {
                    id,
                    kind: kind.to_string(),
                    author,
                    subject,
                    body: body_text,
                    subreddit: subreddit_val,
                    context,
                    is_new,
                    created_utc,
                });
            }
        }

        Ok(notifications)
    }

    pub fn mark_read(&self, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let token = self.auth.get_token()?;
        let client = reqwest::blocking::Client::new();
        let fullnames: Vec<String> = ids
            .iter()
            .map(|id| format!("t4_{id}"))
            .collect();

        let resp = client
            .post("https://oauth.reddit.com/api/read_message")
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", "linux:redhood:v0.1.0 (by /u/redhood)")
            .form(&[("id", &fullnames.join(","))])
            .send()
            .context("Failed to mark Reddit messages as read")?;

        if !resp.status().is_success() {
            anyhow::bail!(
                "Failed to mark messages as read: {}",
                resp.status().as_u16()
            );
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RedditNotification {
    pub id: String,
    pub kind: String,
    pub author: String,
    pub subject: String,
    pub body: String,
    pub subreddit: String,
    pub context: String,
    pub is_new: bool,
    #[allow(dead_code)]
    pub created_utc: f64,
}
