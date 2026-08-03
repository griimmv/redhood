# RedHood

A Telegram bot daemon that forwards **Reddit inbox messages** and **X/Twitter mentions** to your Telegram DMs via webhooks.

---

## Requirements

- **Rust** 1.85+ (edition 2024)
- A **Telegram bot token** (from [@BotFather](https://t.me/BotFather))
- **Reddit API credentials** (script-type app)
- **X/Twitter API credentials** (project with OAuth 1.0a)
- `ngrok` (for local development) or a public HTTPS endpoint
<br>

## Installation

### Building it yourself

```bash
# Clone or navigate to the project
cd ~/Project/redhood

# Build
cargo build --release

# The binary will be at:
./target/release/redhood
```
<br>

## Configuration

### 1. Fetch the example config

```bash
mkdir -p redhood
curl -L -o redhood/config.toml https://raw.githubusercontent.com/griimmv/redhood/main/config.example.toml
```

### 2. Fill in `redhood/config.toml`

### 3. Environment variables

The Docker setup (docker-compose.yml) sets `RUST_LOG` and `CONFIG_PATH`
itself; you don't need a `.env` file. For bare-metal runs, export them
manually:

```bash
export RUST_LOG=info,redhood=debug
export CONFIG_PATH=redhood/config.toml
```

The default `CONFIG_PATH` is already `redhood/config.toml`, so from the
repo root a plain `cargo run` works without exporting anything.
<br>

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
<br><br>


## Running Locally (with ngrok)

```bash
# Terminal 1: Start ngrok
ngrok http 8080

# Terminal 2: Update redhood/config.toml public_url with the ngrok HTTPS URL
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
sudo cp redhood/config.toml /opt/redhood/

# Edit redhood.service if needed, then install
sudo cp redhood.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now redhood

# View logs
sudo journalctl -u redhood -f
```
<br><br>


## Architecture

```
┌──────────────────────────────────────────────────────┐
│                    RedHood Daemon                    │
│                                                      │
│  ┌──────────┐   ┌──────────────────┐   ┌──────────┐  │
│  │  axum    │   │   teloxide Bot   │   │  Poller  │  │
│  │  Server  │   │   (Dispatcher)   │   │  Loop    │  │
│  │  :8080   │   │                  │   │          │  │
│  └────┬─────┘   └────────┬─────────┘   └────┬─────┘  │
│       │                  │                  │        │
│       │ Webhook          │ DM to owner      │ Poll   │
│       ▼                  ▼                  ▼        │
│  ┌──────────┐      ┌──────────┐    ┌──────────────┐  │
│  │ Telegram │      │ Telegram │    │ Reddit API   │  │
│  │ Updates  │      │ Messages │    │ X/Twitter API│  │
│  └──────────┘      └──────────┘    └──────────────┘  │
│                                                      │
│  ┌──────────────────────────────────────────────┐    │
│  │              SQLite (state.db)               │    │
│  │  - last_seen IDs (dedup)                     │    │
│  │  - sent notifications log                    │    │
│  └──────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────┘
```
<br>

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

### Webhook Reception

```
Telegram ──POST /webhook──▶ ngrok ──▶ axum server ──▶ teloxide Dispatcher
                                                          │
                                                    ┌─────▼─────┐
                                                    │  Command   │
                                                    │  Handler   │
                                                    └───────────┘
```

<br>

## Telegram Bot Commands

- `/start` – Welcome and overview
- `/status` – Show current config and state
- `/pause` – Pause all polling
- `/resume` – Resume polling
- `/help` – List commands
<br>

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
│   └── inbox.rs       # Fetch unread inbox, mark as read after
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
- [ ] Create actual `redhood/config.toml` from example
- [ ] Test Reddit API connection
- [ ] Test Twitter API connection
- [ ] Test Telegram webhook via ngrok
- [ ] End-to-end: receive notification and see it in Telegram
- [ ] Write deployment guide (systemd install steps)
- [ ] Clean up: remove dead code, improve error handling
- [ ] Run `cargo clippy` and address warnings
