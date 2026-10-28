# Public applications

If your app makes requests for **more than one server** (a Discord bot many servers
invite, a dashboard, a CAD), the ER:LC API requires a public application with a global API
key. Requests for several servers without one are refused with code `2005`.

## Register your app

1. Create an application on the [ER:LC API dashboard](https://api.erlc.gg/developers/applications).
2. Keep its description, feature list, privacy policy and terms of service up to date.
3. Copy the **global API key**. It is shown only once.

See [Creating public applications](https://apidocs.erlc.gg/creating-public-applications).

## One client, many servers

Build one client with the global key, then derive a client per server with
`with_server_key`. They share the connection pool and rate-limit state.

```rust,no_run
use std::collections::HashMap;

use erlc::Client;

fn build(global_key: String, server_keys: HashMap<u64, String>) -> erlc::Result<HashMap<u64, Client>> {
    let mut keys = server_keys.into_iter();
    let Some((first_guild, first_key)) = keys.next() else {
        return Ok(HashMap::new());
    };

    let base = Client::builder()
        .server_key(first_key)
        .global_key(global_key)
        .user_agent("my-bot/1.0")
        .build()?;

    let mut clients = HashMap::from([(first_guild, base.clone())]);
    for (guild, key) in keys {
        clients.insert(guild, base.with_server_key(key)?);
    }
    Ok(clients)
}
```

`GET` requests made with a global key share one `global` rate limit across every server
your app serves. Plan polling intervals with that in mind.

## Commands from public apps

Server owners may need to authorise your app before it can run commands on their server,
using an [authorization link](https://apidocs.erlc.gg/creating-authorization-links):

```text
https://api.erlc.gg/server-owners/server/[INTERNAL_SERVER_ID]/authorize/[YOUR_APPS_ID]
```

Generate these links during onboarding. This crate does not create them for you.

## Responsibilities

From the [acceptable use policy](https://apidocs.erlc.gg/policies/aup):

- Store server keys securely, like passwords.
- Stop using a server's key as soon as your app is removed from that server.
- If a key keeps returning 403 (`ApiError::is_auth()`), stop using it.
- Support your own users; the ER:LC team only supports developers.
- Apps with high traffic must have proper rate limiting, error handling and abuse
  prevention.
