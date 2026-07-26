use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use teloxide::requests::Requester;
use teloxide::types::ChatId;
use teloxide::Bot;

use crate::format;
use crate::reddit::auth::RedditAuth;
use crate::reddit::inbox::RedditInbox;
use crate::twitter::auth::TwitterAuth;
use crate::twitter::mentions::TwitterMentions;
use crate::AppState;

pub async fn run(state: Arc<AppState>, bot: Bot) {
    let mut handles = Vec::new();

    if let Some(ref reddit_cfg) = state.config.reddit {
        let auth = RedditAuth::new(
            &reddit_cfg.client_id,
            &reddit_cfg.client_secret,
            &reddit_cfg.username,
            &reddit_cfg.password,
        );
        let interval = Duration::from_secs(reddit_cfg.poll_interval_secs);
        let state = state.clone();
        let bot = bot.clone();
        handles.push(tokio::spawn(async move {
            reddit_poll_loop(state, bot, auth, interval).await;
        }));
    }

    if let Some(ref twitter_cfg) = state.config.twitter {
        let auth = TwitterAuth::new(
            &twitter_cfg.api_key,
            &twitter_cfg.api_secret_key,
            &twitter_cfg.access_token,
            &twitter_cfg.access_token_secret,
        );
        let interval = Duration::from_secs(twitter_cfg.poll_interval_secs);
        let state = state.clone();
        let bot = bot.clone();
        handles.push(tokio::spawn(async move {
            twitter_poll_loop(state, bot, auth, interval).await;
        }));
    }

    if handles.is_empty() {
        tracing::warn!("No pollers configured — add [reddit] and/or [twitter] sections to config.toml");
        return;
    }

    tracing::info!("Poller started with {} source(s)", handles.len());

    for handle in handles {
        let _ = handle.await;
    }
}

async fn reddit_poll_loop(state: Arc<AppState>, bot: Bot, auth: RedditAuth, interval: Duration) {
    let mut timer = tokio::time::interval(interval);
    loop {
        timer.tick().await;
        if !state.paused.load(Ordering::SeqCst) {
            if let Err(e) = poll_reddit(&state, &bot, &auth).await {
                tracing::error!("Reddit poll error: {e:#}");
            }
        }
    }
}

async fn twitter_poll_loop(state: Arc<AppState>, bot: Bot, auth: TwitterAuth, interval: Duration) {
    let mut timer = tokio::time::interval(interval);
    loop {
        timer.tick().await;
        if !state.paused.load(Ordering::SeqCst) {
            if let Err(e) = poll_twitter(&state, &bot, &auth).await {
                tracing::error!("Twitter poll error: {e:#}");
            }
        }
    }
}

async fn poll_reddit(
    state: &Arc<AppState>,
    bot: &Bot,
    auth: &RedditAuth,
) -> anyhow::Result<()> {
    let auth_for_blocking = auth.clone();
    let notifications = tokio::task::spawn_blocking(move || {
        let inbox = RedditInbox::new(&auth_for_blocking);
        inbox.fetch_unread()
    })
    .await??;

    let new_notifications: Vec<_> = notifications
        .into_iter()
        .filter(|n| n.is_new)
        .collect();

    if new_notifications.is_empty() {
        return Ok(());
    }

    let mut sent_ids: Vec<(String, String)> = Vec::new();

    for notif in &new_notifications {
        let dedup_key = format!("reddit_{}", notif.id);
        let already_sent = state.db.is_notification_sent(&dedup_key)?;

        if already_sent {
            continue;
        }

        let text = format::format_reddit_message(&notif.kind, &serde_json::json!({
            "author": &notif.author,
            "subject": &notif.subject,
            "body": &notif.body,
            "subreddit": &notif.subreddit,
            "context": &notif.context,
        }));

        let owner = ChatId(state.config.telegram.owner_chat_id);
        match bot.send_message(owner, &text).await {
            Ok(_) => {
                state.db.mark_notification_sent(&dedup_key, "reddit")?;
                sent_ids.push((notif.kind.clone(), notif.id.clone()));
                tracing::info!("Sent Reddit notification: {}", notif.id);
            }
            Err(e) => {
                tracing::error!("Failed to send Reddit notification: {e}");
            }
        }
    }

    if !sent_ids.is_empty() {
        let auth_clone = auth.clone();
        tokio::task::spawn_blocking(move || {
            let inbox = RedditInbox::new(&auth_clone);
            inbox.mark_read(&sent_ids)
        })
        .await??;
    }

    Ok(())
}

async fn poll_twitter(
    state: &Arc<AppState>,
    bot: &Bot,
    auth: &TwitterAuth,
) -> anyhow::Result<()> {
    let user_id = state
        .config
        .twitter
        .as_ref()
        .map(|t| t.user_id.clone())
        .unwrap_or_default();

    let since_id = state.db.get_state("twitter_since_id")?;
    let mentions = TwitterMentions::new(auth, &user_id);

    let results = mentions.fetch_mentions(since_id.as_deref()).await?;

    if results.is_empty() {
        return Ok(());
    }

    let mut all_successful = true;

    for tweet in &results {
        let dedup_key = format!("twitter_{}", tweet.id);
        let already_sent = state.db.is_notification_sent(&dedup_key)?;

        if already_sent {
            continue;
        }

        let text = format::format_tweet(
            &tweet.text,
            &tweet.username,
            &tweet.display_name,
            &tweet.id,
        );

        let owner = ChatId(state.config.telegram.owner_chat_id);
        match bot.send_message(owner, &text).await {
            Ok(_) => {
                state.db.mark_notification_sent(&dedup_key, "twitter")?;
                tracing::info!("Sent tweet notification: {}", tweet.id);
            }
            Err(e) => {
                tracing::error!("Failed to send tweet notification: {e}");
                all_successful = false;
            }
        }
    }

    if all_successful
        && let Some(newest) = results.first()
    {
        state.db.set_state("twitter_since_id", &newest.id)?;
    }

    Ok(())
}
