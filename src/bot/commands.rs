use std::sync::atomic::Ordering;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::requests::ResponseResult;
use teloxide::types::ChatId;

use crate::AppState;

pub async fn handle_message(bot: Bot, msg: Message, state: Arc<AppState>) -> ResponseResult<()> {
    let owner_id = ChatId(state.config.telegram.owner_chat_id);
    if msg.chat.id != owner_id {
        return Ok(());
    }

    let text = match msg.text() {
        Some(t) => t,
        None => return Ok(()),
    };

    match text {
        "/start" => cmd_start(bot, msg, state).await,
        "/status" => cmd_status(bot, msg, state).await,
        "/pause" => cmd_pause(bot, msg, state).await,
        "/resume" => cmd_resume(bot, msg, state).await,
        "/help" => cmd_help(bot, msg).await,
        _ => {
            let _ = bot.send_message(msg.chat.id, "Unknown command. Try /help").await?;
        }
    }

    Ok(())
}

async fn cmd_start(bot: Bot, msg: Message, _state: Arc<AppState>) {
    let text = "\u{1F43B} Welcome to RedHood!\n\n\
        I forward your Reddit inbox and X/Twitter mentions here.\n\n\
        Commands:\n\
        /status - Show bot status\n\
        /pause  - Pause notifications\n\
        /resume - Resume notifications\n\
        /help   - Show this message";

    let _ = bot.send_message(msg.chat.id, text).await;
}

async fn cmd_status(bot: Bot, msg: Message, state: Arc<AppState>) {
    let paused = if state.paused.load(Ordering::SeqCst) { "PAUSED" } else { "ACTIVE" };
    let reddit_interval = state.config.reddit.poll_interval_secs;
    let twitter_interval = state.config.twitter.poll_interval_secs;

    let text = format!(
        "\u{2139} RedHood Status\n\n\
        Status: {paused}\n\
        Reddit poll: every {reddit_interval}s\n\
        Twitter poll: every {twitter_interval}s\n\
        Owner chat: {}\n\
        DB: {}",
        state.config.telegram.owner_chat_id,
        state.config.database.path,
    );

    let _ = bot.send_message(msg.chat.id, text).await;
}

async fn cmd_pause(bot: Bot, msg: Message, state: Arc<AppState>) {
    state.paused.store(true, Ordering::SeqCst);
    let _ = bot.send_message(msg.chat.id, "\u{23F8} Notifications paused").await;
}

async fn cmd_resume(bot: Bot, msg: Message, state: Arc<AppState>) {
    state.paused.store(false, Ordering::SeqCst);
    let _ = bot.send_message(msg.chat.id, "\u{25B6} Notifications resumed").await;
}

async fn cmd_help(bot: Bot, msg: Message) {
    let text = "\u{2753} Commands:\n\
        /start  - Welcome message\n\
        /status - Bot status\n\
        /pause  - Pause notifications\n\
        /resume - Resume notifications\n\
        /help   - This message";

    let _ = bot.send_message(msg.chat.id, text).await;
}
