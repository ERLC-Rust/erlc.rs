# Rate limits

The ER:LC API limits how many requests you can send. The full rules are in the
[official rate limit docs](https://apidocs.erlc.gg/rate-limits); this page explains what
the client does for you.

## Buckets

Every response says which bucket it counted against, in `X-RateLimit-Bucket`:

| Bucket | Applies to |
|---|---|
| `global` | All requests with a global API key, per application |
| `unauthenticated-global` | Requests without a global key, per IP address |
| `unauthenticated-server` | Requests without a global key, per server key |
| `command` | `POST /command`, documented as 1 request per 5 seconds |
| `invalid-requests`, `invalid-requests-server` | Blocks after too many failing requests |

Limits can change and can differ per application, so the client reads them from response
headers instead of hard-coding them.

## What the client does

1. **Tracks headers.** It records `X-RateLimit-Limit`, `-Remaining` and `-Reset` per bucket.
2. **Stops after a 429.** The API requires clients to stop until `Retry-After` has passed.
   Until then, requests to the same bucket fail immediately with `Error::RateLimited`
   without being sent, and `is_local()` returns `true`.
3. **Stops at zero remaining.** When a bucket reports 0 requests left, the client also waits
   for its reset time (ignored if more than 5 minutes away, to guard against clock skew).
4. **Never retries.** Waiting and retrying is your decision.

Respecting `Retry-After` matters: clients that keep sending after a 429 get longer blocks,
and persistent offenders are blocked by Cloudflare.

## Reading the current limits

```rust,no_run
async fn show_limits(client: &erlc::Client) -> erlc::Result<()> {
    client.server_info().await?;
    for bucket in ["global", "unauthenticated-global", "command"] {
        if let Some(info) = client.rate_limit(bucket) {
            println!("{bucket}: {:?}/{:?} left, resets {:?}", info.remaining, info.limit, info.reset);
        }
    }
    Ok(())
}
```

## Waiting and retrying reads

For reads, waiting once and trying again is safe:

```rust,no_run
use erlc::Error;

async fn players_with_wait(client: &erlc::Client) -> erlc::Result<Vec<erlc::models::Player>> {
    match client.players().await {
        Err(Error::RateLimited(limit)) => {
            let wait = limit.retry_after().unwrap_or(std::time::Duration::from_secs(5));
            tokio::time::sleep(wait).await;
            client.players().await
        }
        other => other,
    }
}
```

Do not do this blindly for commands; see [Commands](commands.md).

## Several servers

Clients created with `with_server_key` share rate-limit state:

- `global`, `unauthenticated-global` and `invalid-requests` are shared by every server,
  because they limit your whole application or IP address.
- `command` and the per-server buckets are tracked separately for each server key, so a
  command cooldown on one server does not block another.

The ER:LC docs do not state the scope of the `command` bucket; per-server tracking is this
crate's assumption.

## Tips

- Fetch several datasets with one `server_with` call instead of several shortcuts.
- Poll on an interval (for example every 5-10 seconds), not in a tight loop.
- Use one shared `Client` so all tasks see the same limits.
- Do not rotate IP addresses or use proxies to avoid limits; it is against the
  [acceptable use policy](https://apidocs.erlc.gg/policies/aup).
- Shared-IP hosting such as BotGhost is not supported by the ER:LC API.
