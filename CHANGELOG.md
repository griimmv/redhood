# Changelog

All notable changes to this project will be documented in this file.
## [v0.1.0] - 2026-08-03


### Housekeeping

- Automate releases with cargo-release


### Refactoring

- Namespace config in redhood/ and drop .env
## [v0.3.3] - 2026-07-30


### Bug Fixes

- Add explicit read-only permissions to version and binaries jobs


### Refactoring

- Move blocking filesystem scan off async executor
- Avoid symlink recursion


### Ci

- Disable credential persistence on all actions/checkout steps


### Version

- Bump
## [v0.3.2] - 2026-07-30


### Bug Fixes

- Add filter so it skips .dockerbuild
## [v0.3.1] - 2026-07-30


### Bug Fixes

- Switch to rustls and bump all GH Actions to Node.js 24-compatible versions
## [v0.3.0] - 2026-07-30


### Features

- Added /video command


### Testing

- Add unit tests for video.rs
## [v0.2.3] - 2026-07-30


### Bug Fixes

- Change to use taiki for tool installation for ci cd
## [v0.2.2] - 2026-07-30


### Bug Fixes

- Fall back to /r/{subreddit} when context is an empty string


### Housekeeping

- Replace dead conventional_commits table with commit_parsers, remove unsupported Tera preprocessor


### Ci

- Set workflow-level read-only token permissions
- Idk something something
- Rename binaries before upload to prevent merge-multiple collision
- Run container as non-root user
## [v0.2.1] - 2026-07-29


### Bug Fixes

- Test failing fix
## [v0.2.0] - 2026-07-29


### Bug Fixes

- Resolve Reddit mark_read prefix, Twitter since_id data loss, and client reuse
- Fixed something
- Some edit on cicd


### Features

- Make Reddit and Twitter credentials optional with /status warnings
- Add coordinated graceful shutdown via watch channel
- Add exponential backoff retry to Reddit and Twitter polls
- Add GET /health endpoint to webhook server
- Add CI/CD pipeline with version bumping on tag push


### Housekeeping

- Add changelog generation with git-cliff


### Testing

- Add unit tests for config, db, format, and auth modules
- Test adjust


### Bot

- Await poller shutdown, signal poller on dispatcher exit
