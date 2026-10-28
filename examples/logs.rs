//! Prints recent kill, command and mod call logs: `ERLC_SERVER_KEY=... cargo run --example logs`

use erlc::{Client, ServerQuery};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::var("ERLC_SERVER_KEY")
        .map_err(|_| "set the ERLC_SERVER_KEY environment variable to your server key")?;
    let client = Client::new(key)?;

    let snapshot = client
        .server_with(ServerQuery::new().kill_logs().command_logs().mod_calls())
        .await?;

    println!("Kills:");
    for kill in snapshot.kill_logs.unwrap_or_default() {
        println!(
            "  [{}] {} killed {}",
            kill.timestamp.as_secs(),
            kill.killer.name(),
            kill.killed.name()
        );
    }

    println!("Commands:");
    for log in snapshot.command_logs.unwrap_or_default() {
        println!(
            "  [{}] {}: {}",
            log.timestamp.as_secs(),
            log.player.name(),
            log.command
        );
    }

    println!("Mod calls:");
    for call in snapshot.mod_calls.unwrap_or_default() {
        let handler = call.moderator.as_ref().map_or("unanswered", |m| m.name());
        println!(
            "  [{}] {} ({handler})",
            call.timestamp.as_secs(),
            call.caller.name()
        );
    }

    Ok(())
}
