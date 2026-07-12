# RedHood

A Telegram bot daemon that forwards **Reddit inbox messages** and **X/Twitter mentions** to your Telegram DMs via webhooks.

---

## Requirements

- **Rust** 1.85+ (edition 2024)
- A **Telegram bot token** (from [@BotFather](https://t.me/BotFather))
- **Reddit API credentials** (script-type app)
- **X/Twitter API credentials** (project with OAuth 1.0a)
- `ngrok` (for local development) or a public HTTPS endpoint

## Installation

```bash
# Clone or navigate to the project
cd ~/Project/redhood

# Build
cargo build --release

# The binary will be at:
./target/release/redhood
```

## Configuration

### 1. Copy the example config

```bash
cp config.example.toml config.toml
```

### 2. Fill in `config.toml`

```toml
[telegram]
bot_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
owner_chat_id = 123456789       # Your Telegram user ID

[reddit]
client_id = "YOUR_CLIENT_ID"
client_secret = "YOUR_CLIENT_SECRET"
username = "YOUR_REDDIT_USERNAME"
password = "YOUR_REDDIT_PASSWORD"
poll_interval_secs = 60

[twitter]
api_key = "YOUR_API_KEY"
api_secret_key = "YOUR_API_SECRET"
access_token = "YOUR_ACCESS_TOKEN"
access_token_secret = "YOUR_ACCESS_TOKEN_SECRET"
user_id = "YOUR_USER_ID"        # Numeric X/Twitter user ID
poll_interval_secs = 60

[database]
path = "redhood.db"

[webhook]
host = "0.0.0.0"
port = 8080
public_url = "https://your-ngrok-url.ngrok.io"
```

### 3. Set up environment (optional)

```bash
# .env file
RUST_LOG=info,redhood=debug
CONFIG_PATH=config.toml
```

## Getting API Credentials

### Telegram
1. Message [@BotFather](https://t.me/BotFather) on Telegram
2. Run `/newbot` and follow instructions
3. Copy the bot token
4. Find your user ID by messaging [@userinfobot](https://t.me/userinfobot)

### Reddit (Script App)
1. Go to https://www.reddit.com/prefs/apps
2. Click **create another app**
3. Choose **script**
4. Note the client ID (under the app name) and client secret
5. Use your Reddit username and password

### X/Twitter
1. Go to https://developer.twitter.com/ and create a project
2. Enable OAuth 1.0a with Read permissions
3. Generate API Key + Secret and Access Token + Secret
4. Find your numeric user ID (use a tool like https://tweeterid.com)

## Running Locally (with ngrok)

```bash
# Terminal 1: Start ngrok
ngrok http 8080

# Terminal 2: Update config.toml public_url with the ngrok HTTPS URL
# Then start the bot
RUST_LOG=debug cargo run
```

## Running as a Daemon (systemd)

```bash
# Build release binary
cargo build --release

# Copy files to /opt/redhood
sudo mkdir -p /opt/redhood
sudo cp target/release/redhood /opt/redhood/
sudo cp config.toml /opt/redhood/

# Edit redhood.service if needed, then install
sudo cp redhood.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now redhood

# View logs
sudo journalctl -u redhood -f
```

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                    RedHood Daemon                     │
│                                                      │
│  ┌──────────┐   ┌──────────────────┐   ┌──────────┐  │
│  │  axum    │   │   teloxide Bot   │   │  Poller  │  │
│  │  Server  │   │   (Dispatcher)   │   │  Loop    │  │
│  │  :8080   │   │                  │   │          │  │
│  └────┬─────┘   └────────┬─────────┘   └────┬─────┘  │
│       │                  │                   │        │
│       │ Webhook          │ DM to owner       │ Poll   │
│       ▼                  ▼                   ▼        │
│  ┌──────────┐     ┌──────────┐     ┌──────────────┐   │
│  │ Telegram │     │ Telegram │     │ Reddit API   │   │
│  │ Updates  │     │ Messages │     │ X/Twitter API│   │
│  └──────────┘     └──────────┘     └──────────────┘   │
│                                                      │
│  ┌──────────────────────────────────────────────┐    │
│  │              SQLite (state.db)               │    │
│  │  - last_seen IDs (dedup)                     │    │
│  │  - sent notifications log                    │    │
│  └──────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────┘
```

## Data Flow / Workflow

### Polling Loop (runs every N seconds)

```
1. Poller wakes up
       │
       ├── Reddit: GET /api/v1/message/inbox
       │     │
       │     ├── Check `is_new` field
       │     ├── Dedup against SQLite
       │     ├── Format as Telegram message
       │     ├── Send DM to owner
       │     └── Mark as read on Reddit
       │
       └── Twitter: GET /2/users/:id/mentions
             │
             ├── Check `since_id`
             ├── Dedup against SQLite
             ├── Format as Telegram message
             ├── Send DM to owner
             └── Update `since_id`

2. Sleep until next interval
```

### Telegram Commands

| Command | Action |
|---|---|
| `/start` | Welcome message |
| `/status` | Show poll intervals, paused state, DB path |
| `/pause` | Stop polling (notifications paused) |
| `/resume` | Resume polling |
| `/help` | List commands |

### Webhook Reception

```
Telegram ──POST /webhook──▶ ngrok ──▶ axum server ──▶ teloxide Dispatcher
                                                          │
                                                    ┌─────▼─────┐
                                                    │  Command   │
                                                    │  Handler   │
                                                    └───────────┘
```

## Telegram Bot Commands

- `/start` – Welcome and overview
- `/status` – Show current config and state
- `/pause` – Pause all polling
- `/resume` – Resume polling
- `/help` – List commands

## Project Structure

```
src/
├── main.rs            # Entry point: load config, init DB, start bot
├── config.rs          # Config file deserialization
├── db.rs              # SQLite: state key/value, sent dedup
├── format.rs          # Format notifications into Telegram text
├── poller.rs          # Scheduled polling loop (Reddit + Twitter)
├── bot/
│   ├── mod.rs         # Webhook setup + Dispatcher wiring
│   └── commands.rs    # /start, /status, /pause, /resume handlers
├── reddit/
│   ├── mod.rs
│   ├── auth.rs        # OAuth2 password grant for Reddit API
│   └── inbox.rs       # Fetch unread inbox, mark as read
└── twitter/
    ├── mod.rs
    ├── auth.rs        # OAuth 1.0a HMAC-SHA1 signing
    └── mentions.rs    # Fetch user mentions via Twitter API v2
```

---

## TODO / Progress

- [x] Project scaffolding (`cargo init`, directory structure)
- [x] Cargo.toml with all dependencies (teloxide, axum, reqwest, rusqlite, etc.)
- [x] Configuration module (`config.rs` + `config.example.toml`)
- [x] SQLite database module (`db.rs`)
- [x] Reddit authentication (OAuth2 password grant) (`reddit/auth.rs`)
- [x] Reddit inbox polling (`reddit/inbox.rs`)
- [x] Twitter OAuth 1.0a signing (`twitter/auth.rs`)
- [x] Twitter mentions polling (`twitter/mentions.rs`)
- [x] Notification formatting (`format.rs`)
- [x] Bot command handlers (`bot/commands.rs`)
- [x] Webhook server setup (`bot/mod.rs`)
- [x] Poller loop orchestration (`poller.rs`)
- [x] Main entry point (`main.rs`)
- [x] systemd service unit (`redhood.service`)
- [x] **Compile and fix all errors** (`cargo check`)
- [ ] Create actual `config.toml` from example
- [ ] Test Reddit API connection
- [ ] Test Twitter API connection
- [ ] Test Telegram webhook via ngrok
- [ ] End-to-end: receive notification and see it in Telegram
- [ ] Write deployment guide (systemd install steps)
- [ ] Clean up: remove dead code, improve error handling
- [ ] Run `cargo clippy` and address warnings
