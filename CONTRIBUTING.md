# Contributing

Thanks for helping improve erlc.rs.

## Setup

Install Rust 1.85 or newer, then:

```sh
cargo test --all-features
```

Tests run against a local mock server and never need an API key.

## Before opening a pull request

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
```

CI runs the same checks on Linux, Windows and macOS, plus an MSRV build and `cargo deny`.

## Guidelines

- **Never commit API keys.** Use environment variables such as `ERLC_SERVER_KEY`.
- Only model behaviour documented at <https://apidocs.erlc.gg/> or observed in a redacted capture. Do not guess endpoints or fields.
- Never add automatic retries for commands.
- Keep comments to short one-liners.
- Add a test for every bug fix and new field.
- Add an entry under `Unreleased` in `CHANGELOG.md`.

## Fixtures

Responses captured from a live server must be redacted before they are committed:
replace player names, Roblox IDs, the server name, owner IDs and the join key.
Keep raw captures in `tests/fixtures/raw/`, which git ignores.
