mod commands;

use crate::AppState;
use std::sync::Arc;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::prelude::*;
use teloxide::update_listeners::webhooks;

pub async fn run(state: Arc<AppState>) -> anyhow::Result<()> {
    let bot = Bot::new(&state.config.telegram.bot_token);

    let addr: std::net::SocketAddr = format!(
        "{}:{}",
        state.config.webhook.host, state.config.webhook.port
    )
    .parse()?;

    let public_url: url::Url = state.config.webhook.public_url.parse()?;
    let options = webhooks::Options::new(addr, public_url);

    let listener = webhooks::axum(bot.clone(), options)
        .await
        .map_err(|e| anyhow::anyhow!("Webhook setup failed: {e:?}"))?;

    let handler = Update::filter_message().endpoint(commands::handle_message);

    let bot_for_poller = bot.clone();
    let state_for_poller = state.clone();
    tokio::spawn(async move {
        crate::poller::run(state_for_poller, bot_for_poller).await;
    });

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .build();

    dispatcher
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;

    Ok(())
}
