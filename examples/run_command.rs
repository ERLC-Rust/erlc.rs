//! Runs one command on the live server: `ERLC_SERVER_KEY=... cargo run --example run_command -- ":h Hi"`

use erlc::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::var("ERLC_SERVER_KEY")
        .map_err(|_| "set the ERLC_SERVER_KEY environment variable to your server key")?;
    let command = std::env::args()
        .nth(1)
        .ok_or("pass the command as the first argument, for example \":h Hello\"")?;
    let client = Client::new(key)?;

    match client.run_command(&command).await {
        Ok(response) => println!("{}", response.message),
        Err(Error::Api(api)) if api.is_server_offline() => {
            eprintln!("The server is offline: commands need at least one player in the server.");
        }
        Err(Error::RateLimited(limit)) => {
            eprintln!(
                "Rate limited; wait {:?} before sending another command.",
                limit.retry_after()
            );
        }
        Err(err) => return Err(err.into()),
    }

    Ok(())
}
