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
    let reddit_auth = RedditAuth::new(
        &state.config.reddit.client_id,
        &state.config.reddit.client_secret,
        &state.config.reddit.username,
        &state.config.reddit.password,
    );

    let twitter_auth = TwitterAuth::new(
        &state.config.twitter.api_key,
        &state.config.twitter.api_secret_key,
        &state.config.twitter.access_token,
        &state.config.twitter.access_token_secret,
    );

    let reddit_interval = Duration::from_secs(state.config.reddit.poll_interval_secs);
    let twitter_interval = Duration::from_secs(state.config.twitter.poll_interval_secs);

    let mut reddit_timer = tokio::time::interval(reddit_interval);
    let mut twitter_timer = tokio::time::interval(twitter_interval);

    tracing::info!("Poller started");

    loop {
        tokio::select! {
            _ = reddit_timer.tick() => {
                if !state.paused.load(Ordering::SeqCst) {
                    if let Err(e) = poll_reddit(&state, &bot, &reddit_auth).await {
                        tracing::error!("Reddit poll error: {e:#}");
                    }
                }
            }
            _ = twitter_timer.tick() => {
                if !state.paused.load(Ordering::SeqCst) {
                    if let Err(e) = poll_twitter(&state, &bot, &twitter_auth).await {
                        tracing::error!("Twitter poll error: {e:#}");
                    }
                }
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
    let since_id = state.db.get_state("twitter_since_id")?;
    let mentions = TwitterMentions::new(auth, &state.config.twitter.user_id);

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
