//! Lists players in the server: `ERLC_SERVER_KEY=... cargo run --example players`

use erlc::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::var("ERLC_SERVER_KEY")
        .map_err(|_| "set the ERLC_SERVER_KEY environment variable to your server key")?;
    let client = Client::new(key)?;

    let players = client.players().await?;
    println!("{} players online", players.len());
    for player in &players {
        let callsign = player.callsign.as_deref().unwrap_or("-");
        println!(
            "{:<20} {:<12} {:<8} {}",
            player.player.name(),
            player.team,
            callsign,
            player.permission.as_str()
        );
    }

    Ok(())
}
