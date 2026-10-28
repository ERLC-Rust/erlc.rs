//! Reads several servers with one global key: `ERLC_GLOBAL_KEY=... ERLC_SERVER_KEYS=a,b cargo run --example public_app`

use erlc::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let global_key = std::env::var("ERLC_GLOBAL_KEY")
        .map_err(|_| "set ERLC_GLOBAL_KEY to your application's global API key")?;
    let server_keys = std::env::var("ERLC_SERVER_KEYS")
        .map_err(|_| "set ERLC_SERVER_KEYS to a comma-separated list of server keys")?;
    let mut keys = server_keys
        .split(',')
        .map(str::trim)
        .filter(|k| !k.is_empty());

    let first_key = keys.next().ok_or("ERLC_SERVER_KEYS is empty")?;
    let base = Client::builder()
        .server_key(first_key)
        .global_key(global_key)
        .user_agent("erlc-rs-public-app-example/0.1")
        .build()?;

    let mut servers = vec![base.clone()];
    for key in keys {
        servers.push(base.with_server_key(key)?);
    }

    for (index, server) in servers.iter().enumerate() {
        match server.server_info().await {
            Ok(info) => println!(
                "#{index}: {} ({}/{} players)",
                info.name, info.current_players, info.max_players
            ),
            Err(Error::Api(api)) if api.is_auth() => {
                // The ER:LC docs advise to stop using a key that keeps failing with 403.
                println!("#{index}: key rejected ({api}); remove it from your configuration");
            }
            Err(Error::RateLimited(limit)) => {
                println!(
                    "#{index}: rate limited, retry after {:?}",
                    limit.retry_after()
                );
            }
            Err(err) => println!("#{index}: {err}"),
        }
    }

    Ok(())
}
