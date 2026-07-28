use std::future::Future;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use teloxide::requests::Requester;
use teloxide::types::ChatId;
use teloxide::Bot;
use tokio::sync::watch;

use crate::format;
use crate::reddit::auth::RedditAuth;
use crate::reddit::inbox::RedditInbox;
use crate::twitter::auth::TwitterAuth;
use crate::twitter::mentions::TwitterMentions;
use crate::AppState;

pub async fn run(state: Arc<AppState>, bot: Bot, shutdown: watch::Receiver<bool>) {
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
        let shutdown = shutdown.clone();
        handles.push(tokio::spawn(async move {
            reddit_poll_loop(state, bot, auth, interval, shutdown).await;
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
        let shutdown = shutdown.clone();
        handles.push(tokio::spawn(async move {
            twitter_poll_loop(state, bot, auth, interval, shutdown).await;
        }));
    }

    if handles.is_empty() {
        tracing::warn!("No pollers configured — add [reddit] and/or [twitter] sections to config.toml");
        return;
    }

    tracing::info!("Poller started with {} source(s)", handles.len());

    for handle in handles {
        if let Err(error) = handle.await {
           tracing::error!("Poller task terminated: {error}");
        }
    }
}


async fn retry_with_backoff<F, Fut, T>(label: &'static str, f: F) -> anyhow::Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    let delays = [Duration::from_secs(1), Duration::from_secs(2)];

    for (i, delay) in delays.iter().enumerate() {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                tracing::warn!("{label} attempt {} failed, retrying in {delay:?}: {e:#}", i + 1);
                tokio::time::sleep(*delay).await;
            }
        }
    }

    let result = f().await;
    if let Err(ref e) = result {
        tracing::error!("{label} failed after {} attempts: {e:#}", delays.len() + 1);
    }
    result
}

async fn reddit_poll_loop(
    state: Arc<AppState>,
    bot: Bot,
    auth: RedditAuth,
    interval: Duration,
    shutdown: watch::Receiver<bool>,
) {
    let mut timer = tokio::time::interval(interval);
    let mut shutdown = shutdown;

    loop {
        tokio::select! {
            _ = timer.tick() => {
                if !state.paused.load(Ordering::SeqCst) {
                    let result = retry_with_backoff("Reddit poll", || poll_reddit(&state, &bot, &auth)).await;
                    if let Err(e) = result {
                        tracing::error!("Reddit poll error: {e:#}");
                    }
                }
            }
            _ = shutdown.changed() => {
                tracing::info!("Reddit poller shutting down");
                break;
            }
        }
    }
}

async fn twitter_poll_loop(
    state: Arc<AppState>,
    bot: Bot,
    auth: TwitterAuth,
    interval: Duration,
    shutdown: watch::Receiver<bool>,
) {
    let mut timer = tokio::time::interval(interval);
    let mut shutdown = shutdown;

    loop {
        tokio::select! {
            _ = timer.tick() => {
                if !state.paused.load(Ordering::SeqCst) {
                    let result = retry_with_backoff("Twitter poll", || poll_twitter(&state, &bot, &auth)).await;
                    if let Err(e) = result {
                        tracing::error!("Twitter poll error: {e:#}");
                    }
                }
            }
            _ = shutdown.changed() => {
                tracing::info!("Twitter poller shutting down");
                break;
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
