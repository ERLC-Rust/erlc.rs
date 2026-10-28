<div class="hero">
<h1>erlc.rs</h1>
<p class="tagline">Async, type-safe Rust SDK for the ER:LC Private Server API.</p>
<p class="buttons">
<a class="primary" href="getting-started.html">Get started</a>
<a href="https://erlc-rust.github.io/erlc.rs/api/erlc/">API reference</a>
<a href="https://github.com/ERLC-Rust/erlc.rs">GitHub</a>
</p>
</div>

<div class="features">
<div><strong>Typed data</strong>Players, staff, logs, vehicles, emergency calls and bans as Rust structs.</div>
<div><strong>One request, many datasets</strong>Pick exactly what you need with <code>ServerQuery</code>.</div>
<div><strong>Safe commands</strong>Sent at most once and never retried.</div>
<div><strong>Rate-limit aware</strong>Blocked requests fail locally instead of hitting the API.</div>
<div><strong>Keys stay secret</strong>Never printed in debug output or error messages.</div>
<div><strong>Public apps</strong>One global key, many servers, shared limits.</div>
</div>

```rust,no_run
use erlc::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(std::env::var("ERLC_SERVER_KEY")?)?;

    let info = client.server_info().await?;
    println!("{}: {}/{} players", info.name, info.current_players, info.max_players);
    Ok(())
}
```

## Guides

| Guide | What it covers |
|---|---|
| [Getting started](getting-started.md) | Installing, getting a server key, your first request |
| [The client](client.md) | `Client`, `ClientBuilder` options, sharing a client, key safety |
| [Server data](server-data.md) | Server status, players, staff, logs, vehicles, emergency calls, bans |
| [Commands](commands.md) | Running in-game commands safely |
| [Errors](errors.md) | Every error type, ER:LC error codes and how to handle them |
| [Rate limits](rate-limits.md) | How the API limits requests and what the client does about it |
| [Public applications](public-apps.md) | Serving many servers with a global API key |
| [Testing your code](testing.md) | Mocking the API, fixtures, live checks |
| [Troubleshooting](troubleshooting.md) | Common problems and fixes |

## Methods at a glance

| Method | ER:LC endpoint | Returns |
|---|---|---|
| `server_info()` | `GET /v2/server` | `ServerInfo` |
| `server_with(query)` | `GET /v2/server?...` | `ServerSnapshot` |
| `players()` | `GET /v2/server?Players=true` | `Vec<Player>` |
| `staff()` | `GET /v2/server?Staff=true` | `Staff` |
| `join_logs()` | `GET /v2/server?JoinLogs=true` | `Vec<JoinLog>` |
| `queue()` | `GET /v2/server?Queue=true` | `Vec<u64>` |
| `kill_logs()` | `GET /v2/server?KillLogs=true` | `Vec<KillLog>` |
| `command_logs()` | `GET /v2/server?CommandLogs=true` | `Vec<CommandLog>` |
| `mod_calls()` | `GET /v2/server?ModCalls=true` | `Vec<ModCall>` |
| `emergency_calls()` | `GET /v2/server?EmergencyCalls=true` | `Vec<EmergencyCall>` |
| `vehicles()` | `GET /v2/server?Vehicles=true` | `Vec<Vehicle>` |
| `bans()` | `GET /v1/server/bans` | `BTreeMap<u64, String>` |
| `run_command(cmd)` | `POST /v2/server/command` | `CommandResponse` |

The full reference for every type and method is at
<https://erlc-rust.github.io/erlc.rs/api/erlc/>, or run `cargo doc --open` in your project.

*Unofficial community project, not affiliated with PRC or Emergency Response: Liberty County.*
