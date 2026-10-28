//! Prints server status: `ERLC_SERVER_KEY=... cargo run --example server_info`

use erlc::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::var("ERLC_SERVER_KEY")
        .map_err(|_| "set the ERLC_SERVER_KEY environment variable to your server key")?;
    let client = Client::new(key)?;

    let info = client.server_info().await?;
    println!("{}", info.name);
    println!("Players: {}/{}", info.current_players, info.max_players);
    println!("Team balance: {}", info.team_balance);

    Ok(())
}
