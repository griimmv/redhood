mod bot;
mod config;
mod db;
mod format;
mod poller;
mod reddit;
mod twitter;
mod video;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::watch;
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

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let mut handle = tokio::spawn(bot::run(state, shutdown_rx));

    tokio::select! {
        result = &mut handle => result??,
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutdown received, stopping...");
            drop(shutdown_tx);
            handle.await??;
        }
    }

    tracing::info!("RedHood stopped");
    Ok(())
}
