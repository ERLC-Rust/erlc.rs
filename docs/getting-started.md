# Getting started

## Requirements

- Rust 1.85 or newer
- An async runtime; the examples use [Tokio](https://tokio.rs)
- An ER:LC private server with the **ERLC API** server pack

## Install

`erlc` is distributed from GitHub. Add it to `Cargo.toml` with a release tag:

```toml
[dependencies]
erlc = { git = "https://github.com/ERLC-Rust/erlc.rs", tag = "v0.1.0" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Pin a `tag` (or `rev`) so builds are reproducible. Cargo does not pick up new releases
from git dependencies automatically: change the tag to upgrade, then run `cargo update -p erlc`.

### TLS features

| Feature | Default | TLS backend |
|---|---|---|
| `rustls` | yes | rustls, pure Rust, no system libraries |
| `native-tls` | no | SChannel on Windows, Secure Transport on macOS, OpenSSL on Linux |

To use native TLS:

```toml
erlc = { git = "https://github.com/ERLC-Rust/erlc.rs", tag = "v0.1.0", default-features = false, features = ["native-tls"] }
```

Disabling both features leaves the client unable to reach the real API over HTTPS.

## Get your server key

1. Join your private server and open **Settings**.
2. Find the **ER:LC API** section and open **API Key**.
3. Copy the key.

The key grants the same access as a server Co-Owner, without the in-game audit trail.
Treat it like a password:

- Load it from an environment variable or a secret store, never from source code.
- Never commit it. Add `.env` files to `.gitignore`.
- Regenerate it in the server settings if it ever leaks.

## Your first request

```rust,no_run
use erlc::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::var("ERLC_SERVER_KEY")?;
    let client = Client::new(key)?;

    let info = client.server_info().await?;
    println!("{}", info.name);
    println!("{}/{} players", info.current_players, info.max_players);

    Ok(())
}
```

Run it with the key in the environment:

```sh
# Linux / macOS
ERLC_SERVER_KEY=your-key cargo run
```

```powershell
# Windows PowerShell: Read-Host keeps the key out of your shell history
$env:ERLC_SERVER_KEY = Read-Host -MaskInput "Server key"
cargo run
```

## Run the bundled examples

Clone the repository and run any example:

```sh
ERLC_SERVER_KEY=your-key cargo run --example server_info
ERLC_SERVER_KEY=your-key cargo run --example players
ERLC_SERVER_KEY=your-key cargo run --example logs
ERLC_SERVER_KEY=your-key cargo run --example run_command -- ":h Hello"
```

`run_command` acts on your live server.

## Next steps

- [The client](client.md) for configuration options
- [Server data](server-data.md) for everything you can read
- [Errors](errors.md) before you ship
