use crate::twitter::auth::TwitterAuth;
use anyhow::{Context, Result};
use serde_json::Value;

pub struct TwitterMentions<'a> {
    auth: &'a TwitterAuth,
    user_id: String,
}

impl<'a> TwitterMentions<'a> {
    pub fn new(auth: &'a TwitterAuth, user_id: &str) -> Self {
        Self {
            auth,
            user_id: user_id.to_string(),
        }
    }

    pub async fn fetch_mentions(&self, since_id: Option<&str>) -> Result<Vec<Tweet>> {
        let mut params: Vec<(&str, &str)> = vec![
            ("max_results", "10"),
            ("tweet.fields", "created_at,author_id"),
            ("user.fields", "username,name,profile_image_url"),
            ("expansions", "author_id"),
        ];

        if let Some(sid) = since_id {
            params.push(("since_id", sid));
        }

        let url = format!(
            "https://api.twitter.com/2/users/{}/mentions",
            self.user_id
        );

        let builder = self
            .auth
            .sign_request(reqwest::Method::GET, &url, &params)
            .context("Failed to sign Twitter request")?;

        let resp = builder
            .send()
            .await
            .context("Failed to fetch Twitter mentions")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Twitter mentions request failed ({}): {body}", status.as_u16());
        }

        let body: Value = resp.json().await?;
        let mut tweets = Vec::new();

        let users = body["includes"]["users"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|u| {
                        let id = u["id"].as_str()?;
                        let username = u["username"].as_str()?;
                        let name = u["name"].as_str().unwrap_or("");
                        Some((id.to_string(), (username.to_string(), name.to_string())))
                    })
                    .collect::<std::collections::HashMap<_, _>>()
            })
            .unwrap_or_default();

        if let Some(data) = body["data"].as_array() {
            for item in data {
                let id = item["id"].as_str().unwrap_or("").to_string();
                let text = item["text"].as_str().unwrap_or("").to_string();
                let author_id = item["author_id"].as_str().unwrap_or("");
                let created_at = item["created_at"].as_str().unwrap_or("");

                let (username, display_name) = users
                    .get(author_id)
                    .cloned()
                    .unwrap_or_else(|| ("unknown".into(), "Unknown".into()));

                tweets.push(Tweet {
                    id,
                    text,
                    username,
                    display_name,
                    author_id: author_id.to_string(),
                    created_at: created_at.to_string(),
                });
            }
        }

        Ok(tweets)
    }
}

#[derive(Debug, Clone)]
pub struct Tweet {
    pub id: String,
    pub text: String,
    pub username: String,
    pub display_name: String,
    #[allow(dead_code)]
    pub author_id: String,
    #[allow(dead_code)]
    pub created_at: String,
}
