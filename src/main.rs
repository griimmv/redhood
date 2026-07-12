mod bot;
mod config;
mod db;
mod format;
mod poller;
mod reddit;
mod twitter;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

pub struct AppState {
    pub config: config::Config,
    pub db: db::Database,
    pub paused: AtomicBool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();

    let cfg = config::Config::load()?;
    let database = db::Database::open(&cfg.database.path)?;
    database.migrate()?;

    let state = Arc::new(AppState {
        config: cfg,
        db: database,
        paused: AtomicBool::new(false),
    });

    tracing::info!("RedHood starting...");

    bot::run(state).await?;

    Ok(())
}
