# erlc.rs

Async, type-safe Rust SDK for the [ER:LC Private Server API](https://apidocs.erlc.gg/).

- Typed models for server status, players, staff, logs, emergency calls, vehicles and bans
- One request for several datasets with `ServerQuery`
- Commands sent exactly once, never retried
- Follows ER:LC rate limits: blocked requests fail locally instead of being sent
- Keys are never printed in `Debug` output or error messages
- Public-application support with a global API key and many server keys

## Installation

The crate is distributed from GitHub:

```toml
[dependencies]
erlc = { git = "https://github.com/ERLC-Rust/erlc.rs", tag = "v0.1.0" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

TLS uses rustls by default. To use the platform's native TLS instead:

```toml
erlc = { git = "https://github.com/ERLC-Rust/erlc.rs", tag = "v0.1.0", default-features = false, features = ["native-tls"] }
```

## Quick start

Get your server key in-game under **Private Server settings → ER:LC API → API Key**, then:

```rust,no_run
use erlc::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(std::env::var("ERLC_SERVER_KEY")?)?;

    let info = client.server_info().await?;
    println!("{}: {}/{} players", info.name, info.current_players, info.max_players);

    for player in client.players().await? {
        println!("{} ({})", player.player.name(), player.team);
    }
    Ok(())
}
```

> **Keep your server key secret.** It grants Co-Owner-level access without the in-game
> audit trail. Load it from the environment or a secret store, never commit it, and
> regenerate it if it leaks.

## Usage

### Several datasets in one request

```rust,no_run
use erlc::ServerQuery;

async fn kills(client: &erlc::Client) -> erlc::Result<()> {
    let snapshot = client
        .server_with(ServerQuery::new().players().kill_logs().vehicles())
        .await?;

    for kill in snapshot.kill_logs.unwrap_or_default() {
        println!("{} killed {}", kill.killer.name(), kill.killed.name());
    }
    Ok(())
}
```

Shortcuts fetch one dataset each: `players`, `staff`, `join_logs`, `queue`, `kill_logs`,
`command_logs`, `mod_calls`, `emergency_calls`, `vehicles` and `bans`.

### Commands

```rust,no_run
async fn announce(client: &erlc::Client) -> erlc::Result<()> {
    let response = client.run_command(":h Server restart in 5 minutes").await?;
    println!("{}", response.message);
    Ok(())
}
```

Commands are sent once and never retried. If a command times out, it may still have run.
The ER:LC policy forbids using `:pm` as a replacement for in-game chat.

### Errors

```rust,no_run
use erlc::{Error, ErrorCode};

async fn hint(client: &erlc::Client) {
    match client.run_command(":h hi").await {
        Ok(_) => {}
        Err(Error::Api(e)) if e.is_server_offline() => eprintln!("server has no players"),
        Err(Error::Api(e)) if e.code() == Some(ErrorCode::RestrictedCommand) => eprintln!("restricted"),
        Err(Error::RateLimited(e)) => eprintln!("retry after {:?}", e.retry_after()),
        Err(e) => eprintln!("{e}"),
    }
}
```

### Rate limits

After a `429`, the client refuses to send requests to the same bucket until the
`Retry-After` delay has passed, and returns `Error::RateLimited` with `is_local() == true`.
Nothing is retried automatically. `client.rate_limit("global")` returns the last limits
the API reported.

### Public applications

Apps that serve more than one server must use a
[global API key](https://apidocs.erlc.gg/creating-public-applications):

```rust,no_run
fn clients() -> erlc::Result<(erlc::Client, erlc::Client)> {
    let base = erlc::Client::builder()
        .server_key("first-server-key")
        .global_key("your-global-key")
        .user_agent("my-bot/1.0")
        .build()?;
    let other = base.with_server_key("second-server-key")?;
    Ok((base, other))
}
```

Clients created with `with_server_key` share the connection pool and rate-limit state.

## Examples

```sh
ERLC_SERVER_KEY=your-key cargo run --example server_info
ERLC_SERVER_KEY=your-key cargo run --example players
ERLC_SERVER_KEY=your-key cargo run --example logs
ERLC_SERVER_KEY=your-key cargo run --example run_command -- ":h Hello"
ERLC_GLOBAL_KEY=your-global-key ERLC_SERVER_KEYS=key-one,key-two cargo run --example public_app
```

## Documentation

- [Guides](docs/README.md): getting started, server data, commands, errors, rate limits, public apps, testing and troubleshooting
- [API reference](https://erlc-rust.github.io/erlc.rs/), or `cargo doc --open` locally

## Minimum supported Rust version

Rust 1.85 (edition 2024).

## Disclaimer

This is an unofficial community project. It is not affiliated with or endorsed by
Police Roleplay Community (PRC) or Emergency Response: Liberty County.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
