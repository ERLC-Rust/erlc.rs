# The client

## Creating a client

`Client::new` creates a client for one server with default settings:

```rust,no_run
fn create() -> erlc::Result<erlc::Client> {
    erlc::Client::new(std::env::var("ERLC_SERVER_KEY").unwrap_or_default())
}
```

It fails with `Error::InvalidInput` if the key is empty or contains characters that
cannot be sent in an HTTP header. Surrounding whitespace, such as a trailing newline from
a file, is trimmed.

## Builder options

Use `Client::builder()` for anything beyond the defaults:

```rust,no_run
use std::time::Duration;

fn create() -> erlc::Result<erlc::Client> {
    erlc::Client::builder()
        .server_key(std::env::var("ERLC_SERVER_KEY").unwrap_or_default())
        .timeout(Duration::from_secs(15))
        .user_agent("my-bot/1.0")
        .build()
}
```

| Method | Default | Purpose |
|---|---|---|
| `server_key(key)` | required | Private server key, sent as `Server-Key` |
| `global_key(key)` | none | Global API key for [public applications](public-apps.md), sent as `Authorization` |
| `timeout(duration)` | 10 seconds | Limit for each request, including reading the body |
| `user_agent(name)` | `erlc-rs/<version>` | Identifies your app; the crate's identifier is appended |
| `base_url(url)` | `https://api.erlc.gg` | Point at a proxy or a [mock server](testing.md); a path prefix is kept |
| `http_client(client)` | built-in | Your own `erlc::reqwest::Client`, e.g. with a proxy |

`build()` returns `Error::InvalidInput` for a missing or invalid key, an invalid base URL
or user agent, or a zero timeout.

### Custom HTTP client

```rust,no_run
fn create() -> erlc::Result<erlc::Client> {
    let http = erlc::reqwest::Client::builder()
        .pool_max_idle_per_host(4)
        .build()
        .expect("valid reqwest configuration");

    erlc::Client::builder()
        .server_key(std::env::var("ERLC_SERVER_KEY").unwrap_or_default())
        .http_client(http)
        .build()
}
```

The builder's timeout, user agent and authentication headers are applied to every request
even with a custom client. `erlc::reqwest` is re-exported so the versions always match.

## Sharing a client

`Client` is cheap to clone and safe to share between tasks. Clones share the connection
pool and [rate-limit state](rate-limits.md), so create one client at startup and pass
clones around instead of building a new one per request:

```rust,no_run
async fn spawn_workers(client: erlc::Client) {
    let a = client.clone();
    let b = client.clone();
    let (players, staff) = tokio::join!(a.players(), b.staff());
    println!("{:?} {:?}", players.map(|p| p.len()), staff.map(|s| s.admins.len()));
}
```

## Several servers

`with_server_key` returns a client for another server that shares the same settings,
connection pool and rate-limit state. Apps that call the API for more than one server
must also set a global key; see [Public applications](public-apps.md).

## Keys are never printed

Keys are marked as sensitive and never appear in `Debug` output or error messages:

```rust,no_run
fn show(client: &erlc::Client) {
    // Prints the base URL, timeout and whether a global key is set; never a key.
    println!("{client:?}");
}
```

Responses can still contain server data such as the join key, so be deliberate about
what you log.
