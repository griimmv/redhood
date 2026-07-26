use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub telegram: TelegramConfig,
    pub reddit: Option<RedditConfig>,
    pub twitter: Option<TwitterConfig>,
    pub database: DatabaseConfig,
    pub webhook: WebhookConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub owner_chat_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditConfig {
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
    pub poll_interval_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TwitterConfig {
    pub api_key: String,
    pub api_secret_key: String,
    pub access_token: String,
    pub access_token_secret: String,
    pub user_id: String,
    pub poll_interval_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookConfig {
    pub host: String,
    pub port: u16,
    pub public_url: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".into());
        let content = std::fs::read_to_string(&path)?;
        let cfg: Self = toml::from_str(&content)?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        let tg = &self.telegram;
        anyhow::ensure!(!tg.bot_token.is_empty(), "telegram.bot_token must not be empty");
        anyhow::ensure!(tg.owner_chat_id > 0, "telegram.owner_chat_id must be positive");
        anyhow::ensure!(!Self::is_placeholder(&tg.bot_token), "telegram.bot_token is still a placeholder value");
        if let Some(ref r) = self.reddit {
            anyhow::ensure!(r.poll_interval_secs > 0, "reddit.poll_interval_secs must be > 0");
        }

        if let Some(ref t) = self.twitter {
            anyhow::ensure!(t.poll_interval_secs > 0, "twitter.poll_interval_secs must be > 0");
        }

        anyhow::ensure!(!self.database.path.is_empty(), "database.path must not be empty");

        anyhow::ensure!(!self.webhook.host.is_empty(), "webhook.host must not be empty");
        anyhow::ensure!(self.webhook.port > 0, "webhook.port must be > 0");
        anyhow::ensure!(!self.webhook.public_url.is_empty(), "webhook.public_url must not be empty");
        url::Url::parse(&self.webhook.public_url)
            .map_err(|_| anyhow::anyhow!("webhook.public_url is not a valid URL: {}", self.webhook.public_url))?;

        Ok(())
    }

    pub fn is_placeholder(s: &str) -> bool {
        s.starts_with("YOUR_")
    }

    pub fn telegram_ok(&self) -> bool {
        !Self::is_placeholder(&self.telegram.bot_token)
    }

    pub fn reddit_ok(&self) -> Option<bool> {
        self.reddit.as_ref().map(|r| {
            !Self::is_placeholder(&r.client_id)
                && !Self::is_placeholder(&r.client_secret)
                && !Self::is_placeholder(&r.username)
                && !Self::is_placeholder(&r.password)
        })
    }

    pub fn twitter_ok(&self) -> Option<bool> {
        self.twitter.as_ref().map(|t| {
            !Self::is_placeholder(&t.api_key)
                && !Self::is_placeholder(&t.api_secret_key)
                && !Self::is_placeholder(&t.access_token)
                && !Self::is_placeholder(&t.access_token_secret)
        })
    }
}
