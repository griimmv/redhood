use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u32,
    #[allow(dead_code)]
    token_type: String,
}

pub struct RedditAuth {
    client_id: String,
    client_secret: String,
    username: String,
    password: String,
    token: Mutex<Option<TokenCache>>,
}

#[derive(Clone)]
struct TokenCache {
    access_token: String,
    expires_at: std::time::Instant,
}

impl Clone for RedditAuth {
    fn clone(&self) -> Self {
        Self {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            token: Mutex::new(None),
        }
    }
}

impl RedditAuth {
    pub fn new(client_id: &str, client_secret: &str, username: &str, password: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            token: Mutex::new(None),
        }
    }

    pub fn get_token(&self) -> Result<String> {
        let mut cache = self.token.lock().unwrap();
        if let Some(ref cached) = *cache {
            if cached.expires_at > std::time::Instant::now() {
                return Ok(cached.access_token.clone());
            }
        }

        let token = self.fetch_token()?;
        let expires_at =
            std::time::Instant::now() + std::time::Duration::from_secs(token.expires_in as u64);
        *cache = Some(TokenCache {
            access_token: token.access_token.clone(),
            expires_at,
        });
        Ok(token.access_token)
    }

    fn fetch_token(&self) -> Result<TokenResponse> {
        let auth_bytes = base64_encode(&format!("{}:{}", self.client_id, self.client_secret));
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post("https://www.reddit.com/api/v1/access_token")
            .header("Authorization", format!("Basic {auth_bytes}"))
            .header("User-Agent", "linux:redhood:v0.1.0 (by /u/redhood)")
            .form(&[
                ("grant_type", "password"),
                ("username", &self.username),
                ("password", &self.password),
            ])
            .send()
            .context("Failed to get Reddit access token")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            anyhow::bail!("Reddit token request failed ({}): {body}", status.as_u16());
        }

        let token: TokenResponse = resp.json()?;
        Ok(token)
    }
}

fn base64_encode(input: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input)
}
