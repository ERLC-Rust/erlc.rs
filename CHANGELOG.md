# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `Client` and `ClientBuilder` with server key, global API key, base URL, user agent, custom HTTP client and timeout options.
- `server_info` and `server_with` for `GET /v2/server`, with `ServerQuery` for optional fields.
- Shortcuts: `players`, `staff`, `join_logs`, `queue`, `kill_logs`, `command_logs`, `mod_calls`, `emergency_calls`, `vehicles`.
- `run_command` for `POST /v2/server/command`, sent once and never retried.
- `bans` for `GET /v1/server/bans`.
- Typed models in `erlc::models`.
- Structured errors with `ErrorCode` for every documented ER:LC error code.
- Client-side rate-limit tracking that blocks requests locally after a 429.
- `with_server_key` for public applications serving many servers.
- `rustls` (default) and `native-tls` features.
