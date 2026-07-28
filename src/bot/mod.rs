mod commands;

use crate::AppState;
use std::sync::Arc;
use axum::routing::get;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::prelude::*;
use teloxide::update_listeners::webhooks;
use teloxide::update_listeners::UpdateListener;
use tokio::sync::watch;

pub async fn run(state: Arc<AppState>, mut shutdown: watch::Receiver<bool>) -> anyhow::Result<()> {
    let bot = Bot::new(&state.config.telegram.bot_token);

    let addr: std::net::SocketAddr = format!(
        "{}:{}",
        state.config.webhook.host, state.config.webhook.port
    )
    .parse()?;

    let public_url: url::Url = state.config.webhook.public_url.parse()?;
    let options = webhooks::Options::new(addr, public_url);

    let (mut listener, stop_flag, app) = webhooks::axum_to_router(bot.clone(), options)
        .await
        .map_err(|e| anyhow::anyhow!("Webhook setup failed: {e:?}"))?;

    let app = app.route("/health", get(health_check));

    let stop_token = listener.stop_token();
    let st_bind = stop_token.clone();
    let st_serve = stop_token.clone();
    tokio::spawn(async move {
        let tcp_listener = tokio::net::TcpListener::bind(addr)
            .await
            .inspect_err(|_| st_bind.stop())
            .expect("Couldn't bind to the address");
        axum::serve(tcp_listener, app)
            .with_graceful_shutdown(stop_flag)
            .await
            .inspect_err(|_| st_serve.stop())
            .expect("Axum server error");
    });

    let handler = Update::filter_message().endpoint(commands::handle_message);

    let (poller_shutdown_tx, poller_shutdown_rx) = watch::channel(false);
    let bot_for_poller = bot.clone();
    let state_for_poller = state.clone();
    let poller_handle = tokio::spawn(async move {
        crate::poller::run(state_for_poller, bot_for_poller, poller_shutdown_rx).await;
    });

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .build();

    tokio::select! {
        _ = dispatcher.dispatch_with_listener(listener, LoggingErrorHandler::new()) => {
            stop_token.stop();
        }
        _ = shutdown.changed() => {
            stop_token.stop();
        }
    }

    drop(poller_shutdown_tx);
    if let Err(error) = poller_handle.await {
        tracing::error!("Poller task terminated: {error}");
    }

    Ok(())
}

async fn health_check() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({"status": "ok"}))
}
