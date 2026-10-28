# Commands

`run_command` runs an in-game command as virtual server management, through
`POST /v2/server/command`.

```rust,no_run
async fn announce(client: &erlc::Client) -> erlc::Result<()> {
    let response = client.run_command(":h Server restart in 5 minutes").await?;
    println!("{}", response.message); // usually "Success"
    Ok(())
}
```

- Include the leading `:` as you would in game.
- Surrounding whitespace is trimmed.
- An empty command returns `Error::InvalidInput` without sending anything.

## Delivery guarantee

A command is sent **at most once**. The client never retries it, whatever happens, so a
moderation action such as a ban can never run twice by accident.

The one case to think about is a timeout: the request may have reached the server before
the connection was lost, so **the command may or may not have run**. Check the outcome
(for example with `command_logs()`) before sending it again.

```rust,no_run
use erlc::Error;

async fn kick(client: &erlc::Client, name: &str) -> erlc::Result<()> {
    match client.run_command(format!(":kick {name}")).await {
        Ok(_) => println!("kicked {name}"),
        Err(Error::Transport(e)) if e.is_timeout() => {
            println!("timed out: check the command log before retrying");
        }
        Err(e) => return Err(e),
    }
    Ok(())
}
```

## Rate limit

Commands use their own `command` bucket, which is much stricter than reads: the ER:LC docs
give 1 request per 5 seconds (subject to change). After a 429 the client blocks further
commands for that server until the wait is over. See [Rate limits](rate-limits.md).

Queue commands in your app and send them one at a time rather than firing many at once.

## Errors

| Situation | What you get |
|---|---|
| Server has no players | `Error::Api` with `is_server_offline()` (HTTP 422 or code 3002) |
| Invalid command | `ErrorCode::InvalidCommand` (3001) |
| Restricted command | `ErrorCode::RestrictedCommand` (4002) |
| Prohibited message | `ErrorCode::ProhibitedMessage` (4003) |
| Not authorised | `ErrorCode::Unauthorized` (4000) |
| Too many commands | `Error::RateLimited` |

Some command errors include a `command_id()`. It is only useful when reporting a problem to
the ER:LC API team.

```rust,no_run
use erlc::{Error, ErrorCode};

async fn safe_hint(client: &erlc::Client, text: &str) {
    match client.run_command(format!(":h {text}")).await {
        Ok(_) => {}
        Err(Error::Api(e)) if e.is_server_offline() => eprintln!("nobody is online"),
        Err(Error::Api(e)) if e.code() == Some(ErrorCode::ProhibitedMessage) => {
            eprintln!("message rejected by ER:LC filtering");
        }
        Err(Error::RateLimited(e)) => eprintln!("slow down: {:?}", e.retry_after()),
        Err(e) => eprintln!("command failed: {e}"),
    }
}
```

## Policy

The [ER:LC acceptable use policy](https://apidocs.erlc.gg/policies/aup) applies to every
command you send:

- Do not use `:pm` as a replacement for in-game chat, the phone or the radio.
- No spamming, raiding, impersonation or misinformation.
- Stop sending requests with a server's key as soon as your app is removed from that server.

Public applications may also need the server owner to approve them through an
[authorization link](https://apidocs.erlc.gg/creating-authorization-links) before they can
send commands. See [Public applications](public-apps.md).
