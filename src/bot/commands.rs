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
    use crate::config::Config;

    let paused = if state.paused.load(Ordering::SeqCst) { "PAUSED" } else { "ACTIVE" };

    let reddit_line = match &state.config.reddit {
        Some(cfg) => format!("Reddit poll: every {}s", cfg.poll_interval_secs),
        None => "Reddit poll: not configured".to_string(),
    };

    let twitter_line = match &state.config.twitter {
        Some(cfg) => format!("Twitter poll: every {}s", cfg.poll_interval_secs),
        None => "Twitter poll: not configured".to_string(),
    };

    let credentials_text = {
        let telegram_status = if Config::telegram_ok(&state.config) { "\u{2705}" } else { "\u{26A0}\u{FE0F} placeholders detected" };
        let reddit_status = match state.config.reddit_ok() {
            Some(true) => "\u{2705}".to_string(),
            Some(false) => "\u{26A0}\u{FE0F} placeholders detected".to_string(),
            None => "\u{274C} section missing".to_string(),
        };
        let twitter_status = match state.config.twitter_ok() {
            Some(true) => "\u{2705}".to_string(),
            Some(false) => "\u{26A0}\u{FE0F} placeholders detected".to_string(),
            None => "\u{274C} section missing".to_string(),
        };

        format!(
            "\n\n\
             \u{26A0}\u{FE0F} Credentials\n\n\
             Telegram  {telegram_status}\n\
             Reddit    {reddit_status}\n\
             Twitter   {twitter_status}"
        )
    };

    let text = format!(
        "\u{2139} RedHood Status\n\n\
         Status: {paused}\n\
         {reddit_line}\n\
         {twitter_line}\n\
         Owner chat: {}\n\
         DB: {}\
         {}",
        state.config.telegram.owner_chat_id,
        state.config.database.path,
        credentials_text,
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
