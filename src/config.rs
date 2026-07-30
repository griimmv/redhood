use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub telegram: TelegramConfig,
    pub reddit: Option<RedditConfig>,
    pub twitter: Option<TwitterConfig>,
    pub video: Option<VideoConfig>,
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
pub struct VideoConfig {
    pub base_dir: String,
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

        if let Some(ref v) = self.video {
            anyhow::ensure!(!v.base_dir.is_empty(), "video.base_dir must not be empty");
            let dir = std::path::Path::new(&v.base_dir);
            anyhow::ensure!(dir.exists(), "video.base_dir does not exist: {}", v.base_dir);
            anyhow::ensure!(dir.is_dir(), "video.base_dir is not a directory: {}", v.base_dir);
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

    pub fn video_ok(&self) -> Option<bool> {
        self.video.as_ref().map(|v| {
            !Self::is_placeholder(&v.base_dir)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base_config() -> Config {
        Config {
            telegram: TelegramConfig {
                bot_token: "123456:ABC-DEF1234ghIkl".into(),
                owner_chat_id: 12345,
            },
            reddit: Some(RedditConfig {
                client_id: "client".into(),
                client_secret: "secret".into(),
                username: "user".into(),
                password: "pass".into(),
                poll_interval_secs: 60,
            }),
            twitter: Some(TwitterConfig {
                api_key: "key".into(),
                api_secret_key: "secret".into(),
                access_token: "token".into(),
                access_token_secret: "tokensecret".into(),
                user_id: "123".into(),
                poll_interval_secs: 60,
            }),
            video: None,
            database: DatabaseConfig { path: "redhood.db".into() },
            webhook: WebhookConfig {
                host: "0.0.0.0".into(),
                port: 8080,
                public_url: "https://example.com".into(),
            },
        }
    }

    #[test]
    fn validate_empty_bot_token() {
        let mut cfg = base_config();
        cfg.telegram.bot_token.clear();
        assert!(cfg.validate().unwrap_err().to_string().contains("must not be empty"));
    }

    #[test]
    fn validate_zero_owner_chat() {
        let mut cfg = base_config();
        cfg.telegram.owner_chat_id = 0;
        assert!(cfg.validate().unwrap_err().to_string().contains("must be positive"));
    }

    #[test]
    fn validate_negative_owner_chat() {
        let mut cfg = base_config();
        cfg.telegram.owner_chat_id = -1;
        assert!(cfg.validate().unwrap_err().to_string().contains("must be positive"));
    }

    #[test]
    fn validate_placeholder_bot_token() {
        let mut cfg = base_config();
        cfg.telegram.bot_token = "YOUR_TOKEN".into();
        assert!(cfg.validate().unwrap_err().to_string().contains("placeholder"));
    }

    #[test]
    fn validate_reddit_zero_interval() {
        let mut cfg = base_config();
        cfg.reddit.as_mut().unwrap().poll_interval_secs = 0;
        assert!(cfg.validate().unwrap_err().to_string().contains("must be > 0"));
    }

    #[test]
    fn validate_twitter_zero_interval() {
        let mut cfg = base_config();
        cfg.twitter.as_mut().unwrap().poll_interval_secs = 0;
        assert!(cfg.validate().unwrap_err().to_string().contains("must be > 0"));
    }

    #[test]
    fn validate_empty_db_path() {
        let mut cfg = base_config();
        cfg.database.path.clear();
        assert!(cfg.validate().unwrap_err().to_string().contains("must not be empty"));
    }

    #[test]
    fn validate_zero_port() {
        let mut cfg = base_config();
        cfg.webhook.port = 0;
        assert!(cfg.validate().unwrap_err().to_string().contains("must be > 0"));
    }

    #[test]
    fn validate_empty_webhook_host() {
        let mut cfg = base_config();
        cfg.webhook.host.clear();
        assert!(cfg.validate().unwrap_err().to_string().contains("must not be empty"));
    }

    #[test]
    fn validate_invalid_url() {
        let mut cfg = base_config();
        cfg.webhook.public_url = "not-a-url".into();
        assert!(cfg.validate().unwrap_err().to_string().contains("not a valid URL"));
    }

    #[test]
    fn validate_valid_config() {
        let cfg = base_config();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn is_placeholder_true() {
        assert!(Config::is_placeholder("YOUR_TOKEN"));
    }

    #[test]
    fn is_placeholder_false() {
        assert!(!Config::is_placeholder("abc123"));
    }
}
