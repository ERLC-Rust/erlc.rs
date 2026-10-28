# erlc.rs documentation

A guide to using the `erlc` crate with the [ER:LC Private Server API](https://apidocs.erlc.gg/).

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

## API reference

The full reference for every type and method is generated from the source:

- Online: <https://erlc-rust.github.io/erlc.rs/>
- Locally: `cargo doc --open` in a project that depends on `erlc`

## At a glance

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
